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


from sqlalchemy.ext.asyncio import AsyncSession
from aim.domain.project.repository import Repository
from aim.util import Entity
from aim.util.session_handler import session_handler

__all__ = ["Project"]


class Project(Entity[int]):
    def __init__(
        self, id: int, organization_id: int, name: str, *, repository: Repository
    ):
        super().__init__(id)

        self.organization_id = organization_id
        self.name = name
        self._repository = repository

    async def save(self, session: Optional[AsyncSession] = None):
        """Save the project using a session handler for proper transaction management.
        
        Args:
            session: Optional AsyncSession. If None, a new session will be created and managed.
        """
        async with session_handler(session) as s:
            await self._repository.save(self, session=s)
