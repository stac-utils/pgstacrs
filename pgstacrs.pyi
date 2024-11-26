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

    async def get_version(self) -> str:
        """Returns the pgstac version.

        Returns:
            The pgstac version as a string
        """

    async def get_collection(self, id: str) -> str | None:
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

    async def all_collections(self) -> None:
        """Returns all collections.

        Returns:
            All collections in the database
        """

class PgstacError:
    """An exception returned from pgstac"""
