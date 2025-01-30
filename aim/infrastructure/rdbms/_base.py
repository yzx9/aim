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


from typing import Any, Callable, Concatenate, Coroutine, ParamSpec, TypeVar, cast

from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import DeclarativeBase

from aim.util import AsyncSessionHandler

__all__ = ["Base", "BaseRepository"]

RV = TypeVar("RV")
P = ParamSpec("P")


class Base(DeclarativeBase):
    pass


class BaseRepository:
    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__()
        self._session_handler = session_handler

    def _register(
        self, func: Callable[Concatenate[AsyncSession, P], Coroutine[Any, Any, RV]]
    ) -> Callable[P, Coroutine[Any, Any, RV]]:
        """Decorator to handle session management for repository methods.

        Parameters
        ----------
        func: The repository method to wrap with session handling

        Returns
        -------
        Wrapped async function that handles session management
        """

        async def fn(*args: P.args, **kwargs: P.kwargs) -> RV:
            session = cast(AsyncSession | None, kwargs.pop("session", None))
            async with self._session_handler.session_handler(session) as s:
                return await func(s, *args, **kwargs)

        fn.__name__ = func.__name__
        fn.__doc__ = func.__doc__
        fn.__annotations__ = func.__annotations__
        return fn
