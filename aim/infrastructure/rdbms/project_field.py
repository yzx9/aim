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


from typing import Optional

import sqlalchemy as sa
from sqlalchemy.orm import Mapped, mapped_column

from aim.domain.project.field import FieldData, FieldKind
from aim.infrastructure.rdbms._base import Base, BaseMixin, BaseRepositoryPlus, IntId
from aim.infrastructure.rdbms.project_item import ProjectItemValueModel
from aim.util import AsyncSession, AsyncSessionHandler

__all__ = ["ProjectFieldModel", "ProjectItemRepository"]


class ProjectFieldModel(BaseMixin, Base):
    """SQLAlchemy model representing the project table."""

    __tablename__ = "project_fields"

    project_id: Mapped[IntId]
    name: Mapped[str] = mapped_column(sa.String(64), nullable=False)
    kind: Mapped[str] = mapped_column(sa.String(8), nullable=False)
    default_value_int: Mapped[int] = mapped_column(nullable=True)
    default_value_float: Mapped[float] = mapped_column(nullable=True)
    default_value_string: Mapped[str] = mapped_column(nullable=True)


class ProjectItemRepository(BaseRepositoryPlus[FieldData, ProjectFieldModel]):
    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__(session_handler, ProjectFieldModel)
        self.list_by_project = self._register(self._list_by_project)

    async def _list_by_project(
        self, session: AsyncSession, project_id: int, offset: int, limit: int
    ) -> list[FieldData]:
        """Find all projects for a given organization."""
        stmt = self._list_stmt(offset, limit).where(
            ProjectFieldModel.project_id == project_id
        )
        result = await session.execute(stmt)
        return [self._to_entity(project) for project in result.scalars()]

    def _to_model(
        self, entity: FieldData, model: Optional[ProjectFieldModel] = None
    ) -> ProjectFieldModel:
        if model is None:
            model = ProjectFieldModel()

        model.id = entity.id
        model.project_id = entity.project_id
        model.name = entity.name
        model.kind = entity.kind.value
        (
            model.default_value_int,
            model.default_value_float,
            model.default_value_string,
        ) = ProjectItemValueModel.save_value(entity.kind, entity.default_value)

        return model

    def _to_entity(self, model: ProjectFieldModel) -> FieldData:
        kind = FieldKind(model.kind)
        default_value = ProjectItemValueModel.to_value(
            kind,
            model.default_value_int,
            model.default_value_float,
            model.default_value_string,
        )
        return FieldData(
            id=model.id,
            project_id=model.project_id,
            name=model.name,
            kind=kind,
            default_value=default_value,
        )
