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


from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from aim.domain.user.user import User

__all__ = ["UserRepository"]


class UserRepository(Protocol):
    async def save(self, user: "User") -> None:
        """Save an user to the repository."""
        ...

    async def find(self, id: int) -> "User | None":
        """Find an user by its ID."""
        ...
