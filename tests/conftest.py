from typing import AsyncIterator

import pytest
from pgstacrs import Client
from pytest_postgresql import factories
from pytest_postgresql.executor import PostgreSQLExecutor
from pytest_postgresql.janitor import DatabaseJanitor

pgstac = factories.postgresql_proc(load=["tests.migrate:pgstac"])


@pytest.fixture
async def client(pgstac: PostgreSQLExecutor) -> AsyncIterator[Client]:
    with DatabaseJanitor(
        user=pgstac.user,
        host=pgstac.host,
        port=pgstac.port,
        version=pgstac.version,
        password=pgstac.password,
        dbname="pypgstac_test",
        template_dbname=pgstac.template_dbname,
    ) as database_janitor:
        yield await Client.open(
            f"user={database_janitor.user} host={database_janitor.host} port={database_janitor.port} dbname={database_janitor.dbname} password={database_janitor.password}"
        )
