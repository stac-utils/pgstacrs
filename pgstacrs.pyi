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
        """

    async def print_config(self) -> None:
        """Prints the postgresql configuration.

        Redacts the password
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
