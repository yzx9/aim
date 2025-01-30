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
from sqlalchemy.orm import mapped_column

from aim.domain.project import Project
from aim.infrastructure.rdbms._base import Base, BaseRepository
from aim.util import AsyncSession, AsyncSessionHandler

__all__ = ["ProjectModel", "ProjectRepository"]


class ProjectModel(Base):
    """SQLAlchemy model representing the project table."""

    __tablename__ = "projects"

    id = mapped_column(sa.Integer, primary_key=True, autoincrement=True)
    organization_id = mapped_column(sa.Integer, nullable=False)
    name = mapped_column(sa.String(64), nullable=False)


class ProjectRepository(BaseRepository):
    def __init__(self, session_handler: AsyncSessionHandler) -> None:
        super().__init__(session_handler)
        self.save = self._register(self._save)
        self.find = self._register(self._find)
        self.list_by_organization = self._register(self._list_by_organization)

    async def _save(self, session: AsyncSession, project: Project) -> None:
        """Save a project to the repository."""
        # Check if model exists
        existing = await session.get(ProjectModel, project.id)
        model = self._to_model(project, model=existing)
        if not existing:
            session.add(model)  # Add new model if it doesn't exist

    async def _find(self, session: AsyncSession, id: int) -> Project | None:
        """Find a project by its ID."""
        result = await session.get(ProjectModel, id)
        if not result:
            return None

        return self._to_entity(result)

    async def _list_by_organization(
        self, session: AsyncSession, organization_id: int, offset: int, limit: int
    ) -> list[Project]:
        """Find all projects for a given organization."""
        stmt = (
            sa.select(ProjectModel)
            .where(ProjectModel.organization_id == organization_id)
            .offset(offset)
            .limit(limit)
        )
        result = await session.execute(stmt)
        return [self._to_entity(project) for project in result.scalars()]

    def _to_model(
        self, entity: Project, model: Optional[ProjectModel] = None
    ) -> ProjectModel:
        if model is None:
            model = ProjectModel()

        model.id = entity.id
        model.organization_id = entity.organization_id
        model.name = entity.name
        return model

    def _to_entity(self, model: ProjectModel) -> Project:
        return Project(
            id=model.id,
            organization_id=model.organization_id,
            name=model.name,
            repository=self,
        )
