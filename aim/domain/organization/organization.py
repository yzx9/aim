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


import dataclasses
from typing import Protocol

from aim.util import entity

__all__ = ["Organization", "OrganizationRepository"]


@dataclasses.dataclass
class OrganizationData:
    id: int
    name: str


class OrganizationRepository(Protocol):
    """Protocol for organization repository implementations.

    This protocol defines the interface that all organization repository
    implementations must follow.
    """

    async def save(self, organization: OrganizationData, /) -> None: ...
    async def find(self, id: int) -> OrganizationData | None: ...


@entity
class Organization(OrganizationData):
    def __init__(
        self, data: OrganizationData, *, repository: OrganizationRepository
    ) -> None:
        """Initialize an Organization instance."""
        super().__init__(**dataclasses.asdict(data))
        self._repository = repository

    async def save(self, **kwargs) -> None:
        """Save the organization to the repository.

        This persists the organization using the configured repository.
        """
        await self._repository.save(self, **kwargs)
