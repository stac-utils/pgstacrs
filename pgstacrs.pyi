class Client:
    """A pgstac client."""

    async def get_version(self) -> str:
        """Returns the pgstac version.

        Returns:
            The pgstac version as a string
        """
