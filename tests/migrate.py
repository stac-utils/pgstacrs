from pypgstac.db import PgstacDB
from pypgstac.migrate import Migrate


def pgstac(host: str, port: int, user: str, dbname: str, password: str) -> None:
    pgstac_db = PgstacDB(f"postgresql://{user}:{password}@{host}:{port}/{dbname}")
    Migrate(pgstac_db).run_migration()
