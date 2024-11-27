from typing import Any

import pytest
from pgstacrs import Client
from pytest_postgresql import factories
from pytest_postgresql.executor import PostgreSQLExecutor

pytestmark = pytest.mark.external

pgstac_in_docker = factories.postgresql_noproc(
    host="localhost", port=5432, user="username", password="password", dbname="pgstac"
)


async def test_external(
    pgstac_in_docker: PostgreSQLExecutor,
    collection: dict[str, Any],
    item: dict[str, Any],
) -> None:
    client = await Client.open(
        f"postgresql://{pgstac_in_docker.user}:{pgstac_in_docker.password}@{pgstac_in_docker.host}:{pgstac_in_docker.port}/{pgstac_in_docker.dbname}"
    )
    await client.upsert_collection(collection)
    await client.create_item(item)
    await client.delete_item("20201211_223832_CS2")
