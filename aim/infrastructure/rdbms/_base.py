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
from datetime import datetime
from typing import (
    Annotated,
    Any,
    Callable,
    Concatenate,
    Coroutine,
    Generic,
    Optional,
    ParamSpec,
    Protocol,
    Type,
    TypeVar,
    cast,
)

import sqlalchemy as sa
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column

from aim.util import AsyncSessionHandler

__all__ = ["Base", "BaseMixin", "IntId", "BaseRepository", "BaseRepositoryPlus"]


class Base(DeclarativeBase):
    """Base class for SQLAlchemy declarative models.

    This class serves as the foundation for all SQLAlchemy ORM models in the
    application. It provides the necessary metaclass and registry for declarative
    model definitions.
    """

    pass


class BaseEntity(Protocol):
    id: int


IntId = Annotated[int, mapped_column(sa.BigInteger, nullable=False)]


class BaseMixin:
    """SQLAlchemy model representing the table."""

    id: Mapped[IntId] = mapped_column(primary_key=True)
    utc_created: Mapped[datetime] = mapped_column(
        nullable=False, default=datetime.now, index=True
    )
    utc_updated: Mapped[datetime] = mapped_column(
        nullable=False, default=datetime.now, onupdate=datetime.now
    )
    # soft delete if not null
    utc_deleted: Mapped[datetime] = mapped_column(nullable=True, index=True)


E = TypeVar("E", bound=BaseEntity)
M = TypeVar("M", bound=BaseMixin)

RV = TypeVar("RV")
P = ParamSpec("P")


class BaseRepository(ABC):
    """Base repository class providing session utilities."""

    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__()
        self._session_handler = session_handler

    def _register(
        self, func: Callable[Concatenate[AsyncSession, P], Coroutine[Any, Any, RV]]
    ) -> Callable[P, Coroutine[Any, Any, RV]]:
        """Decorator to handle session management for repository methods.

        Parameters
        ----------
        func : The repository method to wrap with session handling

        Returns
        -------
        fn : Wrapped async function that handles session management
        """

        async def fn(*args: P.args, **kwargs: P.kwargs) -> RV:
            session = cast(AsyncSession | None, kwargs.pop("session", None))
            async with self._session_handler.session_handler(session) as s:
                return await func(s, *args, **kwargs)

        fn.__name__ = func.__name__
        fn.__doc__ = func.__doc__
        fn.__annotations__ = func.__annotations__
        return fn


class BaseRepositoryPlus(BaseRepository, Generic[E, M]):
    """Base repository class providing common CR operations and utilities.

    This class provides a foundation for repository implementations, handling:
    - Session management
    - Basic CRUD operations
    - Type-safe model/entity conversions
    - Automatic transaction handling

    The repository assumes the following constraints on models:
    - Models must inherit from BaseModel
    - Models must have an integer primary key named 'id'
    - Entities must have an integer 'id' field

    Subclasses must implement:
    - _to_model(): Convert entity to SQLAlchemy model
    - _to_entity(): Convert SQLAlchemy model to entity

    Example:
    ```python
    class MyRepository(BaseRepository[MyEntity, MyModel]):
      def _to_model(self, entity: MyEntity model: Optional[MyModel] = None) -> MyModel:
        # Implementation here
      def _to_entity(self, model: MyModel) -> MyEntity:
        # Implementation here
    ```
    """

    def __init__(self, session_handler: AsyncSessionHandler, model: Type[M]) -> None:
        super().__init__(session_handler)
        self._model = model
        self.save = self._register(self._save)
        self.delete = self._register(self._delete)
        self.find = self._register(self._find)
        self.list = self._register(self._list)

    async def _save(self, session: AsyncSession, entity: E) -> None:
        """Save an organization to the repository."""
        # Check if model exists
        existing = await session.get(self._model, entity.id)
        model = self._to_model(entity, model=existing)
        if not existing:
            session.add(model)  # Add new model if it doesn't exist

    async def _delete(self, session: AsyncSession, id: int) -> None:
        """Delete an entity from the repository."""
        stmt = (
            sa.update(self._model)
            .where(self._model.id == id)
            .where(self._model.utc_deleted.is_(None))
            .values(utc_deleted=datetime.now())
        )
        result = await session.execute(stmt)
        if result.rowcount == 0:
            raise ValueError(f"Entity with ID {id} not found")

    async def _find(self, session: AsyncSession, id: int) -> E | None:
        """Find an entity by its ID."""
        result = await session.get(self._model, id)
        if not result:
            return None

        return self._to_entity(result)

    async def _list(self, session: AsyncSession, offset: int, limit: int) -> list[E]:
        """List all entities in the repository."""
        stmt = sa.select(self._model).offset(offset).limit(limit)
        result = await session.execute(stmt)
        return [self._to_entity(row) for row in result.scalars()]

    @abstractmethod
    def _to_model(self, entity: E, model: Optional[M] = None) -> M: ...

    @abstractmethod
    def _to_entity(self, model: M) -> E: ...
