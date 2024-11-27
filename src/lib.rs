use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyType},
};
use serde_json::Value;
use std::{future::Future, str::FromStr};
use thiserror::Error;
use tokio_postgres::{Config, NoTls};

create_exception!(pgstacrs, PgstacError, PyException);

type PgstacPool = Pool<PostgresConnectionManager<NoTls>>;

macro_rules! pgstac {
    (string $client:expr,$py:expr,$function:expr) => {
        let function = $function.to_string();
        $client.run($py, |pool: PgstacPool| async move {
            let connection = pool.get().await?;
            let query = format!("SELECT pgstac.{}()", function);
            let row = connection.query_one(&query, &[]).await?;
            let value: String = row.try_get(function.as_str())?;
            Ok(value)
        })
    };

    (json $client:expr,$py:expr,$function:expr) => {
        let function = $function.to_string();
        $client.run($py, |pool: PgstacPool| async move {
            let query = format!("SELECT pgstac.{}()", function);
            let connection = pool.get().await?;
            let row = connection.query_one(&query, &[]).await?;
            let value: Value = row.try_get(function.as_str())?;
            Ok(Json(value))
        })
    };

    (option $client:expr,$py:expr,$function:expr,$params:expr) => {
        let function = $function.to_string();
        $client.run($py, |pool: PgstacPool| async move {
            let param_string = (0..$params.len())
                .map(|i| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!("SELECT pgstac.{}({})", function, param_string);
            let connection = pool.get().await?;
            let row = connection.query_one(&query, &$params).await?;
            let value: Option<Value> = row.try_get(function.as_str())?;
            Ok(value.map(Json))
        })
    };

    (void $client:expr,$py:expr,$function:expr,$params:expr) => {
        let function = $function.to_string();
        $client.run($py, |pool: PgstacPool| async move {
            let param_string = (0..$params.len())
                .map(|i| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!("SELECT pgstac.{}({})", function, param_string);
            let connection = pool.get().await?;
            let _ = connection.query_one(&query, &$params).await?;
            Ok(())
        })
    };
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    RunError(#[from] RunError<tokio_postgres::Error>),

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
        let manager = PostgresConnectionManager::new(config.clone(), NoTls);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
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
        pgstac! {
            string self,py,"get_version"
        }
    }

    fn get_collection<'a>(&self, py: Python<'a>, id: String) -> PyResult<Bound<'a, PyAny>> {
        pgstac! {
            option self,py,"get_collection",[&id]
        }
    }

    fn create_collection<'a>(
        &self,
        py: Python<'a>,
        collection: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let collection: Value = pythonize::depythonize(&collection)?;
        pgstac! {
            void self,py,"create_collection",[&collection]
        }
    }

    fn update_collection<'a>(
        &self,
        py: Python<'a>,
        collection: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let collection: Value = pythonize::depythonize(&collection)?;
        pgstac! {
            void self,py,"update_collection",[&collection]
        }
    }

    fn upsert_collection<'a>(
        &self,
        py: Python<'a>,
        collection: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let collection: Value = pythonize::depythonize(&collection)?;
        pgstac! {
            void self,py,"upsert_collection",[&collection]
        }
    }

    fn delete_collection<'a>(&self, py: Python<'a>, id: String) -> PyResult<Bound<'a, PyAny>> {
        pgstac! {
            void self,py,"delete_collection",[&id]
        }
    }

    fn all_collections<'a>(&self, py: Python<'a>) -> PyResult<Bound<'a, PyAny>> {
        pgstac! {
            json self,py,"all_collections"
        }
    }

    #[pyo3(signature = (id, collection_id=None))]
    fn get_item<'a>(
        &self,
        py: Python<'a>,
        id: String,
        collection_id: Option<String>,
    ) -> PyResult<Bound<'a, PyAny>> {
        pgstac! {
            option self,py,"get_item",[&Some(id.as_str()), &collection_id.as_deref()]
        }
    }

    fn create_item<'a>(
        &self,
        py: Python<'a>,
        item: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let item: Value = pythonize::depythonize(&item)?;
        pgstac! {
            void self,py,"create_item",[&item]
        }
    }

    fn create_items<'a>(
        &self,
        py: Python<'a>,
        items: Bound<'a, PyList>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let items: Value = pythonize::depythonize(&items)?;
        pgstac! {
            void self,py,"create_items",[&items]
        }
    }

    fn update_item<'a>(
        &self,
        py: Python<'a>,
        item: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let item: Value = pythonize::depythonize(&item)?;
        pgstac! {
            void self,py,"update_item",[&item]
        }
    }

    fn upsert_item<'a>(
        &self,
        py: Python<'a>,
        item: Bound<'a, PyDict>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let item: Value = pythonize::depythonize(&item)?;
        pgstac! {
            void self,py,"upsert_item",[&item]
        }
    }

    fn upsert_items<'a>(
        &self,
        py: Python<'a>,
        items: Bound<'a, PyList>,
    ) -> PyResult<Bound<'a, PyAny>> {
        let items: Value = pythonize::depythonize(&items)?;
        pgstac! {
            void self,py,"upsert_items",[&items]
        }
    }

    #[pyo3(signature = (id, collection_id=None))]
    fn delete_item<'a>(
        &self,
        py: Python<'a>,
        id: String,
        collection_id: Option<String>,
    ) -> PyResult<Bound<'a, PyAny>> {
        pgstac! {
            void self,py,"delete_item",[&Some(id.as_str()), &collection_id.as_deref()]
        }
    }
}

impl Client {
    fn run<'a, F, T>(
        &self,
        py: Python<'a>,
        f: impl FnOnce(PgstacPool) -> F + Send + 'static,
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
            Error::RunError(err) => PgstacError::new_err(err.to_string()),
            Error::TokioPostgres(err) => PgstacError::new_err(format!("postgres: {err}")),
        }
    }
}

#[pymodule]
fn pgstacrs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add("PgstacError", py.get_type::<PgstacError>())?;
    Ok(())
}
