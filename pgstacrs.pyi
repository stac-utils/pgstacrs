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

class PgstacError:
    """An exception returned from pgstac"""
