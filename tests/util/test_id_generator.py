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


import time
from unittest.mock import patch

import pytest

from aim.util.id_generator import SnowflakeGenerator, UUIDInt64Genearator


def test_uuid_int64_generator_generates_unique_ids():
    generator = UUIDInt64Genearator()
    id1 = generator.generate()
    id2 = generator.generate()
    assert id1 != id2
    assert isinstance(id1, int)
    assert id1.bit_length() <= 64


def test_snowflake_generator_generates_unique_ids():
    generator1 = SnowflakeGenerator(machine_id=1)
    generator2 = SnowflakeGenerator(machine_id=2)
    id1 = generator1.generate()
    id2 = generator2.generate()
    assert id1 != id2
    assert isinstance(id1, int)


def test_snowflake_generator_ids_are_increasing():
    generator = SnowflakeGenerator(machine_id=1)
    id1 = generator.generate()
    id2 = generator.generate()
    assert id1 < id2


def test_snowflake_generator_machine_id_range():
    with pytest.raises(AssertionError):
        SnowflakeGenerator(machine_id=-1)
    with pytest.raises(AssertionError):
        SnowflakeGenerator(machine_id=1024)


def test_snowflake_generator_clock_moved_backwards():
    generator = SnowflakeGenerator(machine_id=1)
    generator.generate()

    # Force the clock to move backwards by mocking time.time
    with patch("time.time") as mocked_time:
        mocked_time.return_value = time.time() - 0.002  # Move time back by 2ms
        with pytest.raises(AssertionError, match="Clock moved backwards"):
            generator.generate()
