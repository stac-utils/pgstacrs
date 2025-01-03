from typing import Any

class Client:
    """A pgstac client."""

    @classmethod
    async def open(cls: type[Client], params: str) -> Client:
        """Opens a new client, backed by a connection pool.

        Args:
            params: The connection parameters, either in `postgresql://` or
                `host=localhost user=username` format.

        Returns:
            A pgstac client

        Examples:
            >>> from pgstacrs import Client
            >>> client = await Client.open("postgresql://username:password@localhost:5432/pgstac")
            >>> client = await Client.open("user=username password=password dbname=pgstac")
        """

    async def search(
        self,
        *,
        intersects: str | dict[str, Any] | None = None,
        ids: str | list[str] | None = None,
        collections: str | list[str] | None = None,
        limit: int | None = None,
        bbox: list[float] | None = None,
        datetime: str | None = None,
        include: str | list[str] | None = None,
        exclude: str | list[str] | None = None,
        sortby: str | list[str] | None = None,
        filter: str | dict[str, Any] | None = None,
        query: dict[str, Any] | None = None,
        **kwargs: str,
    ) -> dict[str, Any]:
        """
        Searches the database with STAC API item search.

        Args:
            collections: Array of one or more Collection IDs that
                each matching Item must be in.
            ids: Array of Item ids to return.
            intersects: Searches items by performing intersection between their
                geometry and provided GeoJSON geometry.
            bbox: Requested bounding box.
            datetime: Single date+time, or a range (`/` separator), formatted to
                RFC 3339, section 5.6.  Use double dots .. for open date ranges.
            include: Fields to include in the response (see [the extension
                docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
                for more on the semantics).
            exclude: Fields to exclude from the response (see [the extension
                docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
                for more on the semantics).
            sortby: Fields by which to sort results (use `-field` to sort descending).
            filter: CQL2 filter expression. Strings will be interpreted as
                cql2-text, dictionaries as cql2-json.
            query: Additional filtering based on properties.
                It is recommended to use filter instead, if possible.
            limit: The page size returned from the server.
            kwargs: Any additional arguments to pass down into the search, e.g a pagination token
        """

    async def print_config(self) -> None:
        """Prints the postgresql configuration.

        Redacts the password
        """

    async def set_setting(self, key: str, value: str) -> None:
        """Sets a pgstac setting.

        Args:
            key: The setting name, e.g. `base_url`
            value: The setting value, e.g. `http://pgstacrs.test`
        """

    async def get_version(self) -> str:
        """Returns the pgstac version.

        Returns:
            The pgstac version as a string
        """

    async def get_collection(self, id: str) -> dict[str, Any] | None:
        """Returns a collection by id, or none if one does not exist.

        Args:
            id: The collection id

        Returns:
            A STAC collection, or None
        """

    async def create_collection(self, collection: dict[str, Any]) -> None:
        """Creates a new collection.

        Args:
            collection: The collection

        Raises:
            PgstacError: If the collection already exists.
        """

    async def update_collection(self, collection: dict[str, Any]) -> None:
        """Updates a collection.

        Args:
            collection: The collection

        Raises:
            PgstacError: If the collection does not exist.
        """

    async def update_collection_extents(self) -> None:
        """Updates all collection extents."""

    async def upsert_collection(self, collection: dict[str, Any]) -> None:
        """Upserts a collection.

        Args:
            collection: The collection
        """

    async def delete_collection(self, id: str) -> None:
        """Deletes a collection by id.

        Args:
            id: The collection id
        """

    async def all_collections(self) -> list[dict[str, Any]]:
        """Returns all collections.

        Returns:
            All collections in the database
        """

    async def get_item(
        self, id: str, collection_id: str | None = None
    ) -> dict[str, Any] | None:
        """Returns an item by id.

        Args:
            id: The item id
            collection_id: The optional collection id

        Returns:
            The item, or None if the item does not exist
        """

    async def create_item(self, item: dict[str, Any]) -> None:
        """Creates an item.

        Args:
            item: The item

        Raises:
            PgstacError: If the item's collection does not exist. The collection
                is determined by the `collection` attribute of the item.
        """

    async def create_items(self, items: list[dict[str, Any]]) -> None:
        """Creates many items.

        Args:
            items: The items

        Raises:
            PgstacError: If the items' collection(s) does not exist.
        """

    async def update_item(self, item: dict[str, Any]) -> None:
        """Updates an item.

        Args:
            item: The item

        Raises:
            PgstacError: If the item does not exist
        """

    async def upsert_item(self, item: dict[str, Any]) -> None:
        """Upserts an item.

        Args:
            item: The item

        Raises:
            PgstacError: If the item's collection does not exist.
        """

    async def upsert_items(self, items: list[dict[str, Any]]) -> None:
        """Upserts many items.

        Args:
            items: The items

        Raises:
            PgstacError: If the items' collection(s) does not exist.
        """

    async def delete_item(self, id: str, collection_id: str | None = None) -> None:
        """Deletes an item by id.

        Args:
            id: The item id
            collection_id: The optional collection id

        Raises:
            PgstacError: If the item cannot be found
        """

class PgstacError:
    """An exception returned from pgstac"""

class StacError:
    """Something doesn't match the STAC specification"""
