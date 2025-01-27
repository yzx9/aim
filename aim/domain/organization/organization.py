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


from typing import Self

from aim.domain.organization.config import generate_id
from aim.domain.organization.repository import repository


class Organization:
    def __init__(self, id: int, name: str) -> None:
        """Initialize an Organization instance.

        Parameters
        ----------
        id : int
            The organization's unique identifier
        name : str
            The organization's name
        """
        self._id = id
        self._name = name

    async def save(self) -> None:
        """Save the organization to the repository.

        This persists the organization instance using the configured repository.
        """
        await repository.save(self)

    @classmethod
    async def new(cls, name: str) -> Self:
        """Create and save a new organization.

        Parameters
        ----------
        name : str
            The name of the new organization

        Returns
        -------
        Self
            A new Organization instance that has been persisted
        """
        id = await generate_id()
        organization = cls(id, name)
        await organization.save()
        return organization

    @classmethod
    async def find(cls, id: int) -> "Organization | None":
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
        return await repository.find(id)
