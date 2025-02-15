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


from collections import defaultdict
from datetime import datetime
from typing import Self

import sqlalchemy as sa
from sqlalchemy.orm import Mapped, mapped_column

from aim.domain.project.field import FieldKind, FieldValue
from aim.domain.project.item import ItemData, ItemValue
from aim.infrastructure.rdbms._base import Base, BaseMixin, BaseRepository, IntId
from aim.util import AsyncSession, AsyncSessionHandler

__all__ = ["ProjectItemRepository"]


#############################
#        Item value         #
#############################


class ProjectItemValueModel(Base):
    __tablename__ = "project_item_values"

    item_id: Mapped[IntId] = mapped_column(primary_key=True)
    field_id: Mapped[IntId] = mapped_column(primary_key=True)

    value_int: Mapped[int] = mapped_column(nullable=True)
    value_float: Mapped[float] = mapped_column(nullable=True)
    value_string: Mapped[str] = mapped_column(nullable=True)

    def to_entity(self, kind: FieldKind) -> ItemValue:
        value = self.to_value(kind, self.value_int, self.value_float, self.value_string)
        return ItemValue(
            field_id=self.field_id,
            kind=kind,
            value=value,
        )

    @classmethod
    def from_entity(cls, item_id: int, entity: ItemValue) -> Self:
        value_int, value_float, value_string = cls.save_value(entity.kind, entity.value)
        return cls(
            item_id=item_id,
            field_id=entity.field_id,
            value_int=value_int,
            value_float=value_float,
            value_string=value_string,
        )

    @staticmethod
    def save_value(kind: FieldKind, value: FieldValue) -> tuple[int, float, str]:
        value_int = 0
        value_float = 0.0
        value_string = ""
        match kind:
            case FieldKind.NUMBER if isinstance(value, float):
                value_float = float(value)

            case FieldKind.NUMBER if isinstance(value, int):
                value_int = int(value)

            case FieldKind.BOOLEAN:
                value_int = 1 if not not value else 0

            case FieldKind.DATETIME:
                value_string = str(value)

            case FieldKind.ENUM if isinstance(value, int):
                value_int = value

            case FieldKind.ENUM if isinstance(value, str):
                value_string = value

            case FieldKind.STRING:
                value_string = str(value)

        return value_int, value_float, value_string

    @staticmethod
    def to_value(
        kind: FieldKind, value_int: int, value_float: float, value_string: str
    ) -> FieldValue:
        match kind:
            case FieldKind.NUMBER:
                return value_float if value_float != 0.0 else value_int

            case FieldKind.BOOLEAN:
                return value_int != 0

            case FieldKind.DATETIME:
                return value_string

            case FieldKind.ENUM:
                return value_int if value_int != 0 else value_string

            case FieldKind.STRING:
                return value_string

            case _:
                return 0


class ProjectItemValueRepository:
    async def list_by_item(
        self, session: AsyncSession, item_id: int
    ) -> list[ProjectItemValueModel]:
        """Find all projects for a given organization."""
        stmt = sa.select(ProjectItemValueModel).where(
            ProjectItemValueModel.item_id == item_id
        )
        result = await session.execute(stmt)
        return list(result.scalars().all())

    async def list_by_items(
        self, session: AsyncSession, item_ids: list[int]
    ) -> dict[int, list[ProjectItemValueModel]]:
        """Find all projects for a given organization."""
        stmt = sa.select(ProjectItemValueModel).where(
            ProjectItemValueModel.item_id.in_(item_ids)
        )
        result = await session.execute(stmt)
        re = defaultdict(list)
        for a in result.scalars():
            re[a.item_id].append(a)

        return re


#############################
#           Item            #
#############################


class ProjectItemModel(BaseMixin, Base):
    __tablename__ = "project_items"

    peoject_id: Mapped[int] = mapped_column(nullable=False)


class ProjectItemRepository(BaseRepository):
    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__(session_handler)
        self._values = ProjectItemValueRepository()
        self.save = self._register(self._save)
        self.list_by_project = self._register(self._list_by_project)

    async def _save(self, session: AsyncSession, entity: ProjectItemModel) -> None:
        """Save an organization to the repository."""

        # TODO: save !!!
        raise NotImplementedError()

        # # Check if model exists
        # existing = await session.get(ItemData, entity.id)
        # model = self._to_model(entity, model=existing)
        # if not existing:
        #     session.add(model)  # Add new model if it doesn't exist

    async def _delete(self, session: AsyncSession, id: int) -> None:
        """Delete an entity from the repository."""
        stmt = (
            sa.update(ProjectItemModel)
            .where(ProjectItemModel.id == id)
            .where(ProjectItemModel.utc_deleted.is_(None))
            .values(utc_deleted=datetime.now())
        )
        result = await session.execute(stmt)
        if result.rowcount == 0:
            raise ValueError(f"Entity with ID {id} not found")

    async def _find(self, session: AsyncSession, id: int) -> ItemData | None:
        """Find an entity by its ID."""
        result = await session.get(ProjectItemModel, id)
        if not result:
            return None

        values = await self._values.list_by_item(session, id)
        return self._to_entity(result, values)

    async def _list(
        self, session: AsyncSession, offset: int, limit: int
    ) -> list[ItemData]:
        """List all entities in the repository."""
        stmt = sa.select(ProjectItemModel).offset(offset).limit(limit)
        result = await session.execute(stmt)
        rows = result.scalars()
        values = await self._values.list_by_items(session, [row.id for row in rows])
        return [self._to_entity(row, values[row.id]) for row in result.scalars()]

    async def _list_by_project(
        self, session: AsyncSession, project_id: int, offset: int, limit: int
    ) -> list[ItemData]:
        """Find all items for a given project."""
        stmt = (
            sa.select(ProjectItemModel)
            .where(ProjectItemModel.peoject_id == project_id)
            .offset(offset)
            .limit(limit)
        )
        result = await session.execute(stmt)
        items = result.scalars()
        values = await self._values.list_by_items(session, [a.id for a in items])
        return [self._to_entity(a, values[a.id]) for a in items]

    def _to_model(
        self, item: ItemData
    ) -> tuple[ProjectItemModel, list[ProjectItemValueModel]]:
        return ProjectItemModel(peoject_id=item.project_id), [
            ProjectItemValueModel.from_entity(item.id, value) for value in item.values
        ]

    def _to_entity(
        self, item: ProjectItemModel, values: list[ProjectItemValueModel]
    ) -> ItemData:
        raise NotImplementedError()
