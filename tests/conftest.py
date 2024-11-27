import json
from pathlib import Path
from typing import Any, AsyncIterator, cast

import pytest
from pgstacrs import Client
from pytest import Config, Parser
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


@pytest.fixture
def collection(examples_path: Path) -> dict[str, Any]:
    with open(examples_path / "collection.json") as f:
        return cast(dict[str, Any], json.load(f))


@pytest.fixture
def item(examples_path: Path) -> dict[str, Any]:
    with open(examples_path / "simple-item.json") as f:
        return cast(dict[str, Any], json.load(f))


@pytest.fixture
def examples_path() -> Path:
    return Path(__file__).parents[1] / "spec-examples" / "v1.0.0"


def pytest_addoption(parser: Parser) -> None:
    parser.addoption(
        "--external",
        action="store_true",
        default=False,
        help="run tests that require an external database via docker compose",
    )


def pytest_collection_modifyitems(config: Config, items: Any) -> None:
    if config.getoption("--external"):
        return
    skip_external = pytest.mark.skip(reason="need --external option to run")
    for item in items:
        if "external" in item.keywords:
            item.add_marker(skip_external)
