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


from aim.util import IdGenerator

__init__ = ["init", "generate_id"]

_id_generator: IdGenerator[int]


def init(id_generator: IdGenerator[int]) -> None:
    global _id_generator
    _id_generator = id_generator


async def generate_id() -> int:
    global _id_generator
    return _id_generator.generate()
