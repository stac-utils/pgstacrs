from typing import Any

import pytest
from pgstacrs import Client, PgstacError


async def test_get_and_create_collection(
    client: Client, collection: dict[str, Any]
) -> None:
    assert await client.get_collection("simple-collection") is None
    await client.create_collection(collection)
    assert await client.get_collection("simple-collection") is not None
    with pytest.raises(PgstacError, match="already exists"):  # type: ignore
        await client.create_collection(collection)


async def test_update_collection(client: Client, collection: dict[str, Any]) -> None:
    with pytest.raises(PgstacError, match="no rows"):  # type: ignore
        await client.update_collection(collection)
    await client.create_collection(collection)
    db_collection = await client.get_collection("simple-collection")
    assert db_collection
    assert db_collection["description"] != "a new description"
    collection["description"] = "a new description"
    await client.update_collection(collection)
    db_collection = await client.get_collection("simple-collection")
    assert db_collection
    assert db_collection["description"] == "a new description"


async def test_upsert_collection(client: Client, collection: dict[str, Any]) -> None:
    await client.upsert_collection(collection)
    db_collection = await client.get_collection("simple-collection")
    assert db_collection
    assert db_collection["description"] != "a new description"
    collection["description"] = "a new description"
    await client.upsert_collection(collection)
    db_collection = await client.get_collection("simple-collection")
    assert db_collection
    assert db_collection["description"] == "a new description"


async def test_delete_collection(client: Client, collection: dict[str, Any]) -> None:
    with pytest.raises(PgstacError, match="no rows"):  # type: ignore
        await client.delete_collection("simple-collection")
    await client.create_collection(collection)
    await client.delete_collection("simple-collection")
    assert await client.get_collection("simple-id") is None


async def test_all_collections(client: Client, collection: dict[str, Any]) -> None:
    assert len(await client.all_collections()) == 0
    await client.create_collection(collection)
    assert len(await client.all_collections()) == 1
    collection["id"] = "just-as-simple-collection"
    await client.create_collection(collection)
    assert len(await client.all_collections()) == 2
    await client.delete_collection("simple-collection")
    assert len(await client.all_collections()) == 1
