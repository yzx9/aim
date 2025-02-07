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

import sqlalchemy as sa
from sqlalchemy.orm import mapped_column

from aim.domain.project.item import Item
from aim.infrastructure.rdbms._base import Base, BaseMixin, BaseRepository
from aim.util import AsyncSession, AsyncSessionHandler

__all__ = ["ProjectItemRepository"]


#############################
#        item value         #
#############################


class ProjectItemValueModel(Base):
    __tablename__ = "project_item_values"

    item_id = mapped_column(sa.Integer, primary_key=True)
    field_id = mapped_column(sa.Integer, primary_key=True)

    value_number = mapped_column(sa.Float, nullable=False, default=0.0)
    value_int = mapped_column(sa.Integer, nullable=True, default=0.0)
    value_text = mapped_column(sa.Float, nullable=True, default=0.0)


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

    peoject_id = mapped_column(sa.Integer, nullable=False)


class ProjectItemRepository(BaseRepository):
    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__(session_handler)
        self._values = ProjectItemValueRepository()
        self.list_by_project = self._register(self._list_by_project)

    async def _list_by_project(
        self, session: AsyncSession, project_id: int, offset: int, limit: int
    ) -> list[Item]:
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

    def _to_entity(
        self, item: ProjectItemModel, values: list[ProjectItemValueModel]
    ) -> Item:
        raise NotImplementedError()
