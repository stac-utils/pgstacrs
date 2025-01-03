#![deny(unused_crate_dependencies)]

use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use pgstac::Pgstac;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyType},
};
use rustls::{ClientConfig, RootCertStore};
use serde_json::Value;
use stac_api::python::{StringOrDict, StringOrList};
use std::{future::Future, str::FromStr};
use thiserror::Error;
use tokio_postgres::{Config, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

create_exception!(pgstacrs, PgstacError, PyException);
create_exception!(pgstacrs, StacError, PyException);

type PgstacPool = Pool<PostgresConnectionManager<MakeRustlsConnect>>;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Geojson(#[from] geojson::Error),

    #[error(transparent)]
    Run(#[from] RunError<tokio_postgres::Error>),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Stac(#[from] stac::Error),

    #[error(transparent)]
    Pgstac(#[from] pgstac::Error),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    #[error(transparent)]
    Pythonize(#[from] pythonize::PythonizeError),

    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[pyclass]
struct Client {
    pool: PgstacPool,
    config: Config,
}

struct Json(Value);

#[pymethods]
impl Client {
    #[classmethod]
    fn open<'a>(
        _: Bound<'_, PyType>,
        py: Python<'a>,
        params: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        let config: Config = params
            .parse()
            .map_err(|err: <Config as FromStr>::Err| PyValueError::new_err(err.to_string()))?;
        let tls_config = ClientConfig::builder()
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth();
        let connector = MakeRustlsConnect::new(tls_config);
        let manager = PostgresConnectionManager::new(config.clone(), connector);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            {
                // Quick connection to get better errors, bb8 will just time out
                let _ = config.connect(NoTls).await.map_err(Error::from)?;
            }
            let pool = Pool::builder().build(manager).await.map_err(Error::from)?; // TODO allow configuration
            {
                let connection = pool.get().await.map_err(Error::from)?;
                connection
                    .execute("SET search_path = pgstac, public", &[])
                    .await
                    .map_err(Error::from)?;
            }
            Ok(Client { pool, config })
        })
    }

    fn print_config(&self) {
        println!("{:?}", self.config);
    }

    fn get_version<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            let value = connection.pgstac_version().await?;
            Ok(value)
        })
    }

    fn set_setting<'a>(
        &self,
        py: Python<'a>,
        key: String,
        value: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.set_pgstac_setting(&key, &value).await?;
            Ok(())
        })
    }

    fn get_collection<'a>(&self, py: Python<'a>, id: String) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            let value = connection.collection(&id).await?;
            Ok(value.map(Json))
        })
    }

    fn create_collection<'a>(
        &self,
        py: Python<'a>,
        collection: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let collection: Value = pythonize::depythonize(&collection)?;
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.add_collection(collection).await?;
            Ok(())
        })
    }

    fn update_collection<'a>(
        &self,
        py: Python<'a>,
        collection: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let collection: Value = pythonize::depythonize(&collection)?;
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.update_collection(collection).await?;
            Ok(())
        })
    }

    fn update_collection_extents<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.update_collection_extents().await?;
            Ok(())
        })
    }

    fn upsert_collection<'a>(
        &self,
        py: Python<'a>,
        collection: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let collection: Value = pythonize::depythonize(&collection)?;
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.upsert_collection(collection).await?;
            Ok(())
        })
    }

    fn delete_collection<'a>(&self, py: Python<'a>, id: String) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.delete_collection(&id).await?;
            Ok(())
        })
    }

    fn all_collections<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            let collections = connection.collections().await?;
            Ok(Json(collections.into()))
        })
    }

    #[pyo3(signature = (id, collection_id=None))]
    fn get_item<'a>(
        &self,
        py: Python<'a>,
        id: String,
        collection_id: Option<String>,
    ) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            let value = connection.item(&id, collection_id.as_deref()).await?;
            Ok(value.map(Json))
        })
    }

    fn create_item<'a>(
        &self,
        py: Python<'a>,
        item: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let item: Value = pythonize::depythonize(&item)?;
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.add_item(item).await?;
            Ok(())
        })
    }

    fn create_items<'a>(
        &self,
        py: Python<'a>,
        items: Bound<'a, PyList>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let items: Value = pythonize::depythonize(&items)?;
        let items = if let Value::Array(items) = items {
            items
        } else {
            return Err(PyValueError::new_err("items is not a list"));
        };
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.add_items(&items).await?;
            Ok(())
        })
    }

    fn update_item<'a>(
        &self,
        py: Python<'a>,
        item: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let item: Value = pythonize::depythonize(&item)?;
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.update_item(item).await?;
            Ok(())
        })
    }

    fn upsert_item<'a>(
        &self,
        py: Python<'a>,
        item: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let item: Value = pythonize::depythonize(&item)?;
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.upsert_item(item).await?;
            Ok(())
        })
    }

    fn upsert_items<'a>(
        &self,
        py: Python<'a>,
        items: Bound<'a, PyList>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let items: Value = pythonize::depythonize(&items)?;
        let items = if let Value::Array(items) = items {
            items
        } else {
            return Err(PyValueError::new_err("items is not a list"));
        };
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection.upsert_items(&items).await?;
            Ok(())
        })
    }

    #[pyo3(signature = (id, collection_id=None))]
    fn delete_item<'a>(
        &self,
        py: Python<'a>,
        id: String,
        collection_id: Option<String>,
    ) -> PyResult<Bound<'a, PyAny>> {
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            connection
                .delete_item(&id, collection_id.as_deref())
                .await?;
            Ok(())
        })
    }

    #[pyo3(signature = (*, collections=None, ids=None, intersects=None, bbox=None, datetime=None, include=None, exclude=None, sortby=None, filter=None, query=None, limit=None, **kwargs))]
    fn search<'a>(
        &self,
        py: Python<'a>,
        collections: Option<StringOrList>,
        ids: Option<StringOrList>,
        intersects: Option<StringOrDict>,
        bbox: Option<Vec<f64>>,
        datetime: Option<String>,
        include: Option<StringOrList>,
        exclude: Option<StringOrList>,
        sortby: Option<StringOrList>,
        filter: Option<StringOrDict>,
        query: Option<Bound<'a, PyDict>>,
        limit: Option<u64>,
        kwargs: Option<Bound<'a, PyDict>>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let search = stac_api::python::search(
            intersects,
            ids,
            collections,
            limit,
            bbox,
            datetime,
            include,
            exclude,
            sortby,
            filter,
            query,
            kwargs,
        )?;
        self.run(py, |pool| async move {
            let connection = pool.get().await?;
            let page = connection.search(search).await?;
            let value = serde_json::to_value(page)?;
            Ok(Json(value))
        })
    }
}

impl Client {
    fn run<'a, F, T>(
        &self,
        py: Python<'a>,
        f: impl FnOnce(Pool<PostgresConnectionManager<MakeRustlsConnect>>) -> F + Send + 'static,
    ) -> PyResult<Bound<'a, PyAny>>
    where
        F: Future<Output = Result<T>> + Send,
        T: for<'py> IntoPyObject<'py>,
    {
        let pool = self.pool.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let value = f(pool).await?;
            Ok(value)
        })
    }
}

impl<'py> IntoPyObject<'py> for Json {
    type Error = pythonize::PythonizeError;
    type Output = Bound<'py, PyAny>;
    type Target = PyAny;
    fn into_pyobject(self, py: Python<'py>) -> std::result::Result<Self::Output, Self::Error> {
        pythonize::pythonize(py, &self.0)
    }
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        match value {
            Error::Stac(err) => StacError::new_err(err.to_string()),
            Error::StacApi(err) => StacError::new_err(err.to_string()),
            Error::Geojson(err) => PyValueError::new_err(format!("geojson: {}", err)),
            Error::SerdeJson(err) => PyValueError::new_err(err.to_string()),
            Error::Pgstac(err) => PgstacError::new_err(err.to_string()),
            Error::Pythonize(err) => PyValueError::new_err(err.to_string()),
            Error::Run(err) => PgstacError::new_err(err.to_string()),
            Error::TokioPostgres(err) => PgstacError::new_err(format!("postgres: {err}")),
        }
    }
}

#[pymodule]
fn pgstacrs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<Client>()?;
    m.add("StacError", py.get_type::<StacError>())?;
    m.add("PgstacError", py.get_type::<PgstacError>())?;
    Ok(())
}
