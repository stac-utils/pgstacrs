from typing import Any

import pytest
from pgstacrs import Client, PgstacError


async def test_get_and_create_item(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    assert await client.get_item("20201211_223832_CS2") is None
    await client.create_collection(collection)
    assert await client.get_item("20201211_223832_CS2") is None
    await client.create_item(item)
    assert await client.get_item("20201211_223832_CS2") is not None
    assert await client.get_item("20201211_223832_CS2", "simple-collection") is not None

    item["collection"] = "does-not-exist"
    with pytest.raises(PgstacError, match="does-not-exist"):  # type: ignore
        await client.create_item(item)


async def test_update_item(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    with pytest.raises(PgstacError, match="query returned no rows"):  # type: ignore
        await client.update_item(item)
    await client.create_collection(collection)
    with pytest.raises(PgstacError, match="query returned no rows"):  # type: ignore
        await client.update_item(item)
    await client.create_item(item)
    item["properties"]["foo"] = "bar"
    await client.update_item(item)
    db_item = await client.get_item("20201211_223832_CS2")
    assert db_item
    assert db_item["properties"]["foo"] == "bar"


async def test_upsert_item(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    with pytest.raises(PgstacError, match="simple-collection"):  # type: ignore
        await client.upsert_item(item)
    await client.create_collection(collection)
    await client.upsert_item(item)
    assert client.get_item("20201211_223832_CS2") is not None
    item["properties"]["foo"] = "bar"
    await client.upsert_item(item)
    # TODO ensure there's only one item
    db_item = await client.get_item("20201211_223832_CS2")
    assert db_item
    assert db_item["properties"]["foo"] == "bar"


async def test_create_items(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.create_items([item])
    assert await client.get_item("20201211_223832_CS2") is not None


async def test_upsert_items(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.upsert_items([item])
    assert await client.get_item("20201211_223832_CS2") is not None
    item["properties"]["foo"] = "bar"
    await client.upsert_items([item])
    db_item = await client.get_item("20201211_223832_CS2")
    assert db_item
    assert db_item["properties"]["foo"] == "bar"


async def test_delete_item(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    with pytest.raises(PgstacError, match="no rows"):  # type: ignore
        await client.delete_item("20201211_223832_CS2")
    with pytest.raises(PgstacError, match="simple-collection"):  # type: ignore
        await client.create_item(item)

    await client.create_collection(collection)
    await client.create_item(item)
    assert await client.get_item("20201211_223832_CS2") is not None
    await client.delete_item("20201211_223832_CS2")
    assert await client.get_item("20201211_223832_CS2") is None

    with pytest.raises(PgstacError, match="no rows"):  # type: ignore
        await client.delete_item("20201211_223832_CS2")

    await client.create_item(item)
    await client.delete_item("20201211_223832_CS2", "simple-collection")
    assert await client.get_item("20201211_223832_CS2") is None


async def test_update_collection_extents(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    collection["extent"]["spatial"]["bbox"] = [[-180, -90, 180, 90]]
    await client.create_collection(collection)
    await client.create_item(item)
    db_collection = await client.get_collection("simple-collection")
    assert db_collection
    assert db_collection["extent"]["spatial"]["bbox"] == [[-180, -90, 180, 90]]
    await client.update_collection_extents()
    db_collection = await client.get_collection("simple-collection")
    assert db_collection
    assert db_collection["extent"]["spatial"]["bbox"] != [[-180, -90, 180, 90]]
