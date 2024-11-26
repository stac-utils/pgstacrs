use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyType},
};
use serde_json::Value;
use std::future::Future;
use thiserror::Error;
use tokio_postgres::NoTls;

create_exception!(pgstacrs, PgstacError, PyException);

type PgstacPool = Pool<PostgresConnectionManager<NoTls>>;

macro_rules! pgstac {
    (string $client:expr,$py:expr,$function:expr) => {
        let function = $function.to_string();
        $client.run($py, |pool: PgstacPool| async move {
            let connection = pool.get().await?;
            let query = format!("SELECT * FROM pgstac.{}()", function);
            let row = connection.query_one(&query, &[]).await?;
            let value: String = row.try_get(function.as_str())?;
            Ok(value)
        })
    };

    (option $client:expr,$py:expr,$function:expr,$params:expr) => {
        let function = $function.to_string();
        $client.run($py, |pool: PgstacPool| async move {
            let param_string = (0..$params.len())
                .map(|i| format!("${}", i + 1))
                .collect::<Vec<_>>()
                .join(", ");
            let query = format!("SELECT * FROM pgstac.{}({})", function, param_string);
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
            let query = format!("SELECT * FROM pgstac.{}({})", function, param_string);
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
struct Client(PgstacPool);

struct Json(Value);

#[pymethods]
impl Client {
    #[classmethod]
    fn open<'a>(
        _: Bound<'_, PyType>,
        py: Python<'a>,
        params: String,
    ) -> PyResult<Bound<'a, PyAny>> {
        let manager =
            PostgresConnectionManager::new_from_stringlike(params, NoTls).map_err(Error::from)?; // TODO enable tls
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let pool = Pool::builder().build(manager).await.map_err(Error::from)?; // TODO allow configuration
            Ok(Client(pool))
        })
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
        let pool = self.0.clone();
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
