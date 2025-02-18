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


import sqlalchemy as sa
from sqlalchemy.orm import Mapped, mapped_column

from aim.domain.project.project import ProjectData
from aim.infrastructure.rdbms._base import Base, BaseMixin, BaseRepositoryPlus, IntId
from aim.util import AsyncSession, AsyncSessionHandler

__all__ = ["ProjectModel", "ProjectRepository"]


class ProjectModel(BaseMixin, Base):
    """SQLAlchemy model representing the project table."""

    __tablename__ = "projects"

    organization_id: Mapped[IntId]
    name: Mapped[str] = mapped_column(sa.String(64), nullable=False)


class ProjectRepository(BaseRepositoryPlus[ProjectData, ProjectModel]):
    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__(session_handler, ProjectModel)
        self.list_by_organization = self._register(self._list_by_organization)

    async def _list_by_organization(
        self, session: AsyncSession, organization_id: int, offset: int, limit: int
    ) -> list[ProjectData]:
        """Find all projects for a given organization."""
        stmt = self._list_stmt(offset, limit).where(
            ProjectModel.organization_id == organization_id
        )
        result = await session.execute(stmt)
        return [self._to_entity(project) for project in result.scalars()]

    def _to_model(
        self, entity: ProjectData, model: ProjectModel | None = None
    ) -> ProjectModel:
        if model is None:
            model = ProjectModel()

        model.id = entity.id
        model.organization_id = entity.organization_id
        model.name = entity.name
        return model

    def _to_entity(self, model: ProjectModel) -> ProjectData:
        return ProjectData(
            id=model.id,
            organization_id=model.organization_id,
            name=model.name,
        )
