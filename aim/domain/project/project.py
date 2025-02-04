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

from aim.util import Entity

__all__ = ["Project", "ProjectRepository"]


class ProjectRepository(Protocol):
    async def save(self, project: "Project", /) -> None:
        """Save an organization to the repository."""
        ...

    async def delete(self, id: int) -> "Project | None":
        """Delete an organization by its ID."""
        ...

    async def find(self, id: int) -> "Project | None":
        """Find an organization by its ID."""
        ...


class Project(Entity[int]):
    def __init__(
        self, id: int, organization_id: int, name: str, *, repository: ProjectRepository
    ):
        super().__init__(id)
        self.organization_id = organization_id
        self.name = name
        self._repository = repository

    async def save(self, **kwargs):
        """Save the project."""
        await self._repository.save(self, **kwargs)

    async def delete(self):
        """Delete the project."""
        await self._repository.delete(self.id)
