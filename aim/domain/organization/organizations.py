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

from aim.domain.organization.organization import (
    Organization,
    OrganizationData,
    OrganizationRepository,
)
from aim.util import IdGenerator, aggregate

__all__ = ["Organizations", "Repository"]


class Repository(Protocol):
    @property
    def organizations(self) -> OrganizationRepository: ...


@aggregate
class Organizations:
    def __init__(self, *, repository: Repository, id_generator: IdGenerator[int]):
        super().__init__()
        self._repository = repository
        self._id_generator = id_generator

    async def new(self, name: str) -> Organization:
        """Create and save a new organization.

        Returns
        -------
        Self
            A new Organization instance that has been persisted
        """
        id = self._id_generator.generate()
        data = OrganizationData(id=id, name=name)
        organization = Organization(data, repository=self._repository.organizations)
        await organization.save()
        return organization

    async def find(self, id: int) -> Organization | None:
        """Find an organization by its ID.

        Parameters
        ----------
        id : int
            The ID of the organization to find

        Returns
        -------
        Organization | None
            The found organization, or None if not found
        """
        data = await self._repository.organizations.find(id)
        if data is None:
            return None

        return Organization(data, repository=self._repository.organizations)
