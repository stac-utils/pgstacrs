import copy
from typing import Any

import pytest
from pgstacrs import Client


async def test_empty_search(client: Client) -> None:
    assert await client.search() == {
        "features": [],
        "links": [
            {"href": ".", "rel": "root", "type": "application/json"},
            {"href": "./search", "rel": "self", "type": "application/json"},
        ],
        "numberReturned": 0,
        "type": "FeatureCollection",
    }


async def test_search(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.create_item(item)
    feature_collection = await client.search()
    assert feature_collection["numberReturned"] == 1


async def test_search_fields(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.create_item(item)

    feature_collection = await client.search(include="id")
    item = feature_collection["features"][0]
    assert item == {"id": "20201211_223832_CS2", "collection": "simple-collection"}

    feature_collection = await client.search(exclude="id")
    item = feature_collection["features"][0]
    assert "id" not in item


@pytest.mark.skip("I'm not sure query is implemented properly in pgstac?")
async def test_search_query(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    item["properties"]["foo"] = "bar"
    await client.create_item(item)

    feature_collection = await client.search(query={"query": {"foo": {"eq": "bar"}}})
    assert feature_collection["numberReturned"] == 1

    feature_collection = await client.search(query={"query": {"foo": {"eq": "baz"}}})
    assert feature_collection["numberReturned"] == 0


async def test_bbox(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.create_item(item)

    feature_collection = await client.search(bbox=[170, 0, 173, 2])
    assert feature_collection["numberReturned"] == 1

    # Looks like my postgres doesn't like 3d bboxes
    # feature_collection = await client.search(bbox=[170, 0, -1000, 173, 2, 20000])
    # assert feature_collection["numberReturned"] == 1

    feature_collection = await client.search(bbox=[0, 0, 1, 1])
    assert feature_collection["numberReturned"] == 0


async def test_sortby(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    item_a = copy.deepcopy(item)
    item_a["id"] = "a"
    item_a["properties"]["foo"] = "a"
    item_a["properties"]["bar"] = 0
    item_b = copy.deepcopy(item)
    item_b["id"] = "b"
    item_b["properties"]["foo"] = "b"
    item_b["properties"]["bar"] = 1
    item_c = copy.deepcopy(item)
    item_c["id"] = "c"
    item_c["properties"]["foo"] = "c"
    item_c["properties"]["bar"] = 1
    await client.create_items([item_a, item_b, item_c])

    feature_collection = await client.search(sortby="+foo")
    assert feature_collection["features"][0]["id"] == "a"
    assert feature_collection["features"][1]["id"] == "b"

    feature_collection = await client.search(sortby="foo")
    assert feature_collection["features"][0]["id"] == "a"
    assert feature_collection["features"][1]["id"] == "b"

    feature_collection = await client.search(sortby="-foo")
    assert feature_collection["features"][0]["id"] == "c"
    assert feature_collection["features"][1]["id"] == "b"

    feature_collection = await client.search(sortby=["-bar", "+foo"])
    assert feature_collection["features"][0]["id"] == "b"
    assert feature_collection["features"][1]["id"] == "c"

    feature_collection = await client.search(sortby=["-bar", "-foo"])
    assert feature_collection["features"][0]["id"] == "c"
    assert feature_collection["features"][1]["id"] == "b"


async def test_filter(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    item["properties"]["foo"] = "bar"
    await client.create_item(item)

    feature_collection = await client.search(filter="foo = 'bar'")
    assert feature_collection["numberReturned"] == 1
    feature_collection = await client.search(filter="foo != 'bar'")
    assert feature_collection["numberReturned"] == 0

    feature_collection = await client.search(
        filter={"op": "=", "args": [{"property": "foo"}, "bar"]}
    )
    assert feature_collection["numberReturned"] == 1
    feature_collection = await client.search(
        filter={"op": "!=", "args": [{"property": "foo"}, "bar"]}
    )
    assert feature_collection["numberReturned"] == 0


async def test_intersects(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.create_item(item)

    feature_collection = await client.search(
        intersects={"type": "Point", "coordinates": [0, 0]}
    )
    assert feature_collection["numberReturned"] == 0

    feature_collection = await client.search(
        intersects={"type": "Point", "coordinates": [172.92, 1.35]}
    )
    assert feature_collection["numberReturned"] == 1

    feature_collection = await client.search(
        intersects='{"type": "Point", "coordinates": [172.92, 1.35]}'
    )
    assert feature_collection["numberReturned"] == 1


async def test_ids(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.create_item(item)

    feature_collection = await client.search(ids="not-an-id")
    assert feature_collection["numberReturned"] == 0

    feature_collection = await client.search(ids="20201211_223832_CS2")
    assert feature_collection["numberReturned"] == 1

    feature_collection = await client.search(ids=["20201211_223832_CS2"])
    assert feature_collection["numberReturned"] == 1


async def test_collections(
    client: Client, collection: dict[str, Any], item: dict[str, Any]
) -> None:
    await client.create_collection(collection)
    await client.create_item(item)

    feature_collection = await client.search(collections="not-an-id")
    assert feature_collection["numberReturned"] == 0

    feature_collection = await client.search(collections="simple-collection")
    assert feature_collection["numberReturned"] == 1

    feature_collection = await client.search(collections=["simple-collection"])
    assert feature_collection["numberReturned"] == 1
