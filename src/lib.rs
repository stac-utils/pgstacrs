use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use pyo3::{
    create_exception,
    exceptions::{PyException, PyIOError},
    prelude::*,
    types::PyType,
};
use std::future::Future;
use thiserror::Error;
use tokio_postgres::NoTls;

create_exception!(pgstacrs, PgstacError, PyException);

type PgstacPool = Pool<PostgresConnectionManager<NoTls>>;

#[pyclass]
struct Client(PgstacPool);

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
        self.string(py, "get_version")
    }
}

impl Client {
    fn string<'a, 'b>(&self, py: Python<'a>, function: &'static str) -> PyResult<Bound<'a, PyAny>>
    where
        'a: 'b,
    {
        let function = function.to_string();
        self.run(py, |pool: PgstacPool| async move {
            let connection = pool.get().await?;
            let query = format!("SELECT * FROM pgstac.{}()", function);
            let row = connection.query_one(&query, &[]).await?;
            let value: String = row.try_get(function.as_str())?;
            Ok(value)
        })
    }

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

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    RunError(#[from] RunError<tokio_postgres::Error>),

    #[error(transparent)]
    Pgstac(#[from] pgstac::Error),

    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
}

type Result<T> = std::result::Result<T, Error>;

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        match value {
            Error::RunError(err) => PgstacError::new_err(err.to_string()),
            Error::Pgstac(err) => PgstacError::new_err(err.to_string()),
            Error::TokioPostgres(err) => PyIOError::new_err(format!("tokio postgres: {err}")),
        }
    }
}

#[pymodule]
fn pgstacrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Client>()?;
    Ok(())
}
