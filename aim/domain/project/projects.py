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


from typing import Protocol

from aim.domain.project.project import Project, ProjectRepository
from aim.util import Aggregate, IdGenerator

__all__ = ["Projects", "Repository"]


class Repository(Protocol):
    @property
    def projects(self) -> ProjectRepository: ...


class Projects(Aggregate[Project, int]):
    def __init__(self, *, repository: Repository, id_generator: IdGenerator[int]):
        super().__init__(id_generator=id_generator)

        self.repository = repository

    async def new(self, organization_id: int, name: str) -> Project:
        """Create and save a new project.

        Parameters
        ----------
        organization_id : int
            The organization id of the new project
        name : str
            The name of the new project

        Returns
        -------
        Self
            A new project that has been persisted
        """
        id = self._id_generator.generate()
        project = Project(
            id, organization_id, name, repository=self.repository.projects
        )
        await project.save()
        return project

    async def find(self, id: int) -> Project | None:
        """Find an project by its ID.

        Parameters
        ----------
        id : int
            The ID of the project to find

        Returns
        -------
        Organization | None
            The found project, or None if not found
        """
        return await self.repository.projects.find(id)
