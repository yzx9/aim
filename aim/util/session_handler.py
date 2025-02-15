# Copyright 2025 Zexin Yuan
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


from abc import ABC, abstractmethod
from collections.abc import AsyncIterator
from contextlib import _AsyncGeneratorContextManager, asynccontextmanager

from sqlalchemy.ext.asyncio import AsyncEngine, AsyncSession, async_sessionmaker

__all__ = ["AsyncSession", "AsyncSessionHandler", "SQLAlchemyAsyncSessionHandler"]


class AsyncSessionHandler(ABC):
    @abstractmethod
    def session_handler(
        self, session: AsyncSession | None = None
    ) -> _AsyncGeneratorContextManager[AsyncSession]: ...


class SQLAlchemyAsyncSessionHandler(AsyncSessionHandler):
    """Manages SQLAlchemy async session creation and configuration."""

    def __init__(self, engine: AsyncEngine):
        super().__init__()
        self._factory = async_sessionmaker(engine, expire_on_commit=False)

    @asynccontextmanager
    async def session_handler(
        self, session: AsyncSession | None = None
    ) -> AsyncIterator[AsyncSession]:
        """Context manager for handling SQLAlchemy async sessions.

        Provides proper transaction handling and error management.
        If no session is provided, creates and manages a new session.

        Examples
        --------
        ```python
        # With existing session
        async with session_handler(session) as s:
            await repo.save(project, session=s)

        # Without existing session
        async with session_handler() as s:
            await repo.save(project, session=s)
        ```
        """

        new_session = None
        if session is None:
            session = self._factory()
            new_session = session

        try:
            yield session
            if new_session is not None:  # Only commit if we created the session
                await session.commit()
        except Exception:
            if new_session is not None:  # Only rollback if we created the session
                await session.rollback()
            raise
        finally:
            if new_session is not None:  # Only close if we created the session
                await session.close()
