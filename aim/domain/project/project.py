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


from aim.domain.project.repository import Repository
from aim.util import Entity

__all__ = ["Project"]


class Project(Entity[int]):
    def __init__(
        self, id: int, organization_id: int, name: str, *, repository: Repository
    ):
        super().__init__(id)
        self.organization_id = organization_id
        self.name = name
        self._repository = repository

    async def save(self, **kwargs):
        """Save the project."""
        await self._repository.save(self, **kwargs)
