use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use geojson::Geometry;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyType},
};
use serde_json::Value;
use stac::Bbox;
use stac_api::{Fields, Filter, Items, Search, Sortby};
use std::{future::Future, str::FromStr};
use thiserror::Error;
use tokio_postgres::{types::ToSql, Config, NoTls};

create_exception!(pgstacrs, PgstacError, PyException);
create_exception!(pgstacrs, StacError, PyException);

type PgstacPool = Pool<PostgresConnectionManager<NoTls>>;

#[derive(FromPyObject)]
pub enum StringOrDict {
    String(String),
    Dict(Py<PyDict>),
}

#[derive(FromPyObject)]
pub enum StringOrList {
    String(String),
    List(Vec<String>),
}

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

    (json $client:expr,$py:expr,$function:expr,$params:expr) => {
        let function = $function.to_string();
        $client.run($py, |pool: PgstacPool| async move {
            let param_string = (0..$params.len())
                .map(|i| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!("SELECT pgstac.{}({})", function, param_string);
            let connection = pool.get().await?;
            let row = connection.query_one(&query, &$params).await?;
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
    Geojson(#[from] geojson::Error),

    #[error(transparent)]
    Run(#[from] RunError<tokio_postgres::Error>),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Stac(#[from] stac::Error),

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
            json self,py,"all_collections",[] as [&(dyn ToSql + Sync); 0]
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

    #[pyo3(signature = (*, collections=None, ids=None, intersects=None, bbox=None, datetime=None, include=None, exclude=None, sortby=None, filter=None, query=None, limit=None))]
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
    ) -> PyResult<Bound<'a, PyAny>> {
        // TODO refactor to use https://github.com/gadomski/stacrs/blob/1528d7e1b7185a86efe9fc7c42b0620093c5e9c6/src/search.rs#L128-L162
        let mut fields = Fields::default();
        if let Some(include) = include {
            fields.include = include.into();
        }
        if let Some(exclude) = exclude {
            fields.exclude = exclude.into();
        }
        let fields = if fields.include.is_empty() && fields.exclude.is_empty() {
            None
        } else {
            Some(fields)
        };
        let query = query
            .map(|query| pythonize::depythonize(&query))
            .transpose()?;
        let bbox = bbox
            .map(|bbox| Bbox::try_from(bbox))
            .transpose()
            .map_err(Error::from)?;
        let sortby = sortby.map(|sortby| {
            Vec::<String>::from(sortby)
                .into_iter()
                .map(|s| s.parse::<Sortby>().unwrap()) // the parse is infallible
                .collect::<Vec<_>>()
        });
        let filter = filter
            .map(|filter| match filter {
                StringOrDict::Dict(cql_json) => {
                    pythonize::depythonize(&cql_json.bind_borrowed(py)).map(Filter::Cql2Json)
                }
                StringOrDict::String(cql2_text) => Ok(Filter::Cql2Text(cql2_text)),
            })
            .transpose()?;
        let filter = filter
            .map(|filter| filter.into_cql2_json())
            .transpose()
            .map_err(Error::from)?;
        let items = Items {
            limit,
            bbox,
            datetime,
            query,
            fields,
            sortby,
            filter,
            ..Default::default()
        };

        let intersects = intersects
            .map(|intersects| match intersects {
                StringOrDict::Dict(json) => pythonize::depythonize(&json.bind_borrowed(py))
                    .map_err(Error::from)
                    .and_then(|json| Geometry::from_json_object(json).map_err(Error::from)),
                StringOrDict::String(s) => s.parse().map_err(Error::from),
            })
            .transpose()?;
        let ids = ids.map(|ids| ids.into());
        let collections = collections.map(|ids| ids.into());
        let search = Search {
            items,
            intersects,
            ids,
            collections,
        };
        let search = serde_json::to_value(search).map_err(Error::from)?;
        pgstac! {
            json self,py,"search",[&search]
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
            Error::Stac(err) => StacError::new_err(err.to_string()),
            Error::StacApi(err) => StacError::new_err(err.to_string()),
            Error::Geojson(err) => PyValueError::new_err(format!("geojson: {}", err)),
            Error::SerdeJson(err) => PyValueError::new_err(err.to_string()),
            Error::Pythonize(err) => PyValueError::new_err(err.to_string()),
            Error::Run(err) => PgstacError::new_err(err.to_string()),
            Error::TokioPostgres(err) => PgstacError::new_err(format!("postgres: {err}")),
        }
    }
}

impl From<StringOrList> for Vec<String> {
    fn from(value: StringOrList) -> Vec<String> {
        match value {
            StringOrList::List(list) => list,
            StringOrList::String(s) => vec![s],
        }
    }
}

#[pymodule]
fn pgstacrs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    m.add("StacError", py.get_type::<StacError>())?;
    m.add("PgstacError", py.get_type::<PgstacError>())?;
    Ok(())
}
