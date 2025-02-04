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

__all__ = ["Organization", "OrganizationRepository"]


class OrganizationRepository(Protocol):
    """Protocol for organization repository implementations.

    This protocol defines the interface that all organization repository
    implementations must follow.
    """

    async def save(self, organization: "Organization", /) -> None:
        """Save an organization to the repository."""
        ...

    async def find(self, id: int) -> "Organization | None":
        """Find an organization by its ID."""
        ...


class Organization(Entity[int]):
    def __init__(
        self, id: int, name: str, *, repository: OrganizationRepository
    ) -> None:
        """Initialize an Organization instance.

        Parameters
        ----------
        id : int
            The organization's unique identifier
        name : str
            The organization's name
        repository : Repository
            The repository used to persist the organization
        """
        super().__init__(id)
        self.name = name
        self._repository = repository

    async def save(self, **kwargs) -> None:
        """Save the organization to the repository.

        This persists the organization using the configured repository.
        """
        await self._repository.save(self, **kwargs)
