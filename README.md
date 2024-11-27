# pgstacrs

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/pgstacrs/ci.yml?style=for-the-badge)](https://github.com/stac-utils/pgstacrs/actions/workflows/ci.yml)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/pgstacrs/docs.yml?style=for-the-badge&label=docs)](https://stac-utils.github.io/pgstacrs)

Python async API for [pgstac](https://github.com/stac-utils/pgstac), backed by Rust.
**pgstacrs** is tested against the following **pgstac** versions:

- [0.9.1](https://github.com/stac-utils/pgstac/releases/tag/v0.9.1)
- [0.8.5](https://github.com/stac-utils/pgstac/releases/tag/v0.8.5)

## Usage

```shell
python -m pip install pgstacrs
```

Then:

```python
from pgstacrs import Client

# Search
client = await Client.open("postgresql://username:password@localhost:5432/pgstac")
feature_collection = await client.search(
    collections=["collection-a"], # or collections="collection-a"
    intersects={"type": "Point", "coordinates": [-105.1019, 40.1672]},
    sortby="-datetime",
)

# CRUD
await client.create_item({"type": "Feature", "id": "foo", ...})
await client.delete_item("foo")
await client.create_items([...])
```

See [the documentation](https://stac-utils.github.io/pgstacrs/) for more.

## Developing

Get [Rust](https://rustup.rs/) and [uv](https://docs.astral.sh/uv/getting-started/installation/).
Then:

```shell
git clone git@github.com:stac-utils/pgstacrs.git
cd pgstacrs
uv sync
scripts/test
```

## License

MIT
