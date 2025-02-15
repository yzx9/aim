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


import ctypes
import datetime
import threading
import time
import uuid
from abc import ABC, abstractmethod
from typing import Generic, TypeVar

__all__ = ["IdGenerator", "SnowflakeGenerator"]

T = TypeVar("T", bound=int | str)


class IdGenerator(ABC, Generic[T]):
    """Abstract base class for ID generators."""

    @abstractmethod
    def generate(self) -> T:
        """Generate a new unique ID.

        Returns
        -------
        int
            A new unique identifier
        """
        # TODO: change to async func
        raise NotImplementedError


class UUIDInt64Genearator(IdGenerator[int]):
    def generate(self) -> int:
        uuid_128 = uuid.uuid4().int
        return ctypes.c_int64(uuid_128).value


# Epoch for Snowflake IDs, default: 2025-01-01 00:00:00 UTC in milliseconds
_DEFAULT_EPOCH = int(
    (datetime.datetime(2025, 1, 1) - datetime.datetime(1970, 1, 1)).total_seconds()
    * 1000
)


class SnowflakeGenerator(IdGenerator[int]):
    def __init__(self, machine_id: int, epoch: int = _DEFAULT_EPOCH) -> None:
        """Generates unique, time-sorted 64-bit IDs using the Snowflake algorithm.

        The ID structure is as follows:
        - 1 bit: unused (sign bit)
        - 41 bits: timestamp in milliseconds since custom epoch
        - 10 bits: machine ID
        - 12 bits: sequence number

        The implementation is thread-safe.

        Parameters
        ----------
        machine_id : int
            Unique ID for this machine/process (0-1023)
        epoch : int, default to 2025-01-01 00:00:00 UTC
            Custom epoch in milliseconds

        Raises
        ------
        AssertionError
            If machine_id is not between 0 and 1023
        """

        assert 0 <= machine_id < 1024, "Machine ID must be between 0 and 1023"

        self._machine_id = machine_id
        self._epoch = epoch
        self._sequence = 0
        self._last_timestamp = -1
        self._lock = threading.Lock()

    def _current_timestamp(self) -> int:
        """Get current timestamp in milliseconds since custom epoch.

        Returns
        -------
        int
            Milliseconds since epoch
        """
        return int(time.time() * 1000) - self._epoch

    def _wait_for_next_millisecond(self, last_timestamp: int) -> int:
        """Wait until next millisecond if we've exhausted sequence numbers.

        Parameters
        ----------
        last_timestamp : int
            The last timestamp used for ID generation

        Returns
        -------
        int
            The next available timestamp
        """

        timestamp = self._current_timestamp()
        while timestamp <= last_timestamp:
            timestamp = self._current_timestamp()
        return timestamp

    def generate(self) -> int:
        """Generate a new Snowflake ID.

        Returns
        -------
        id: int
            A new unique 64-bit Snowflake ID

        Raises
        ------
        ValueError
            If the system clock moves backwards

        Examples
        --------
        >>> generator = SnowflakeGenerator(machine_id=1)
        >>> id1 = generator.generate()
        >>> id2 = generator.generate()
        >>> id3 = generator.generate()
        >>> id1 < id2 < id3
        True
        """

        with self._lock:
            timestamp = self._current_timestamp()
            assert timestamp >= self._last_timestamp, (
                "Clock moved backwards. Refusing to generate ID."
            )

            if timestamp == self._last_timestamp:
                self._sequence = (self._sequence + 1) & 0xFFF
                if self._sequence == 0:
                    timestamp = self._wait_for_next_millisecond(self._last_timestamp)
            else:
                self._sequence = 0

            self._last_timestamp = timestamp

            return (timestamp << 22) | (self._machine_id << 12) | self._sequence
