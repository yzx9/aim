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


import os
from collections.abc import Callable
from io import StringIO
from typing import Any, overload

from dotenv import dotenv_values

__all__ = ["ConfigParser"]


class ConfigParser:
    """Parses configuration from multiple sources with priority.

    Sources are checked in this order (highest to lowest priority):
    1. CLI arguments
    2. Environment variables with a prefix
    3. Multiple .env files
    4. Default values
    """

    def __init__(
        self,
        *,
        cli_args: dict[str, str | int | float | bool] | None = None,
        env_files: list[str | StringIO] | None = None,
        env_prefix: str | None = None,
    ):
        self._cli_args = cli_args or {}
        self._env_prefix = f"{env_prefix.upper()}_" if env_prefix else ""
        self._env_vars = self._load_env_files(env_files or [".env", ".env.local"])

    @overload
    def parse_int(
        self,
        key: str,
        *,
        min_value: int | None = ...,
        max_value: int | None = ...,
    ) -> int | None: ...

    @overload
    def parse_int(
        self,
        key: str,
        default: int,
        *,
        min_value: int | None = ...,
        max_value: int | None = ...,
    ) -> int: ...

    def parse_int(
        self,
        key: str,
        default: int | None = None,
        *,
        min_value: int | None = None,
        max_value: int | None = None,
    ) -> int | None:
        """Parse an integer value with optional range validation.

        Parameters
        ----------
        key : str
            Configuration key to look up
        default : int, optional
            Default value if key not found
        min_value : int, optional
            Minimum allowed value (inclusive)
        max_value : int, optional
            Maximum allowed value (inclusive)

        Returns
        -------
        int or None
            Parsed integer value or default if not found/invalid

        Examples
        --------
        >>> parser = ConfigParser(cli_args={"timeout": "30"})
        >>> parser.parse_int("timeout")
        30
        >>> parser.parse_int("invalid", default=5)
        5
        >>> parser.parse_int("invalid") is None
        True
        """
        value = self._get_value(key)
        if value is None:
            return default

        result = _convert_value(value, int)
        if result is not None:
            if min_value is not None and result < min_value:
                return default

            if max_value is not None and result > max_value:
                return default

        return result

    @overload
    def parse_float(
        self,
        key: str,
        *,
        min_value: float | None = ...,
        max_value: float | None = ...,
    ) -> float | None: ...

    @overload
    def parse_float(
        self,
        key: str,
        default: float,
        *,
        min_value: float | None = ...,
        max_value: float | None = ...,
    ) -> float: ...

    def parse_float(
        self,
        key: str,
        default: float | None = None,
        *,
        min_value: float | None = None,
        max_value: float | None = None,
    ) -> float | None:
        """Parse a float value with optional range validation.

        Parameters
        ----------
        key : str
            Configuration key to look up
        default : float, optional
            Default value if key not found
        min_value : float, optional
            Minimum allowed value (inclusive)
        max_value : float, optional
            Maximum allowed value (inclusive)

        Returns
        -------
        float or None
            Parsed float value or default if not found/invalid

        Examples
        --------
        >>> parser = ConfigParser(cli_args={"ratio": "0.75"})
        >>> parser.parse_float("ratio")
        0.75
        >>> parser.parse_float("missing", default=1.0)
        1.0
        >>> parser.parse_float("missing") is None
        True
        """

        value = self._get_value(key)
        if value is None:
            return default

        result = _convert_value(value, float)
        if result is not None:
            if min_value is not None and result < min_value:
                return default

            if max_value is not None and result > max_value:
                return default

        return result

    @overload
    def parse_str(
        self, key: str, *, allowed_values: list[str] | None = ...
    ) -> str | None: ...
    @overload
    def parse_str(
        self, key: str, default: str, *, allowed_values: list[str] | None = ...
    ) -> str: ...

    def parse_str(
        self,
        key: str,
        default: str | None = None,
        *,
        allowed_values: list[str] | None = None,
    ) -> str | None:
        """Parse a string value with optional allowed values validation.

        Parameters
        ----------
        key : str
            Configuration key to look up
        default : str, optional
            Default value if key not found
        allowed_values : list of str, optional
            List of valid string values

        Returns
        -------
        str or None
            Parsed string value or default if not found/invalid

        Examples
        --------
        >>> parser = ConfigParser(cli_args={"mode": "fast"})
        >>> parser.parse_str("mode")
        'fast'
        >>> parser.parse_str("mode", allowed_values=["slow"]) is None
        True
        >>> parser.parse_str("missing", default="slow")
        'slow'
        >>> parser.parse_str("missing") is None
        True
        """
        value = self._get_value(key)
        if value is None:
            return default

        result = str(value).strip()
        if allowed_values is not None and result not in allowed_values:
            return default

        return result

    @overload
    def parse_bool(self, key: str) -> bool | None: ...
    @overload
    def parse_bool(self, key: str, default: bool) -> bool: ...

    def parse_bool(self, key: str, default: bool | None = None) -> bool | None:
        """Parse a boolean value.

        Parameters
        ----------
        key : str
            Configuration key to look up
        default : bool, optional
            Default value if key not found

        Returns
        -------
        bool or None
            Parsed boolean value or default if not found/invalid

        Examples
        --------
        >>> parser = ConfigParser(cli_args={"debug": "true"})
        >>> parser.parse_bool("debug")
        True
        >>> parser.parse_bool("missing", default=False)
        False
        >>> parser.parse_bool("missing") is None
        True
        """
        value = self._get_value(key)
        if value is None:
            return default

        elif isinstance(value, (int, float)):
            return value > 0

        elif isinstance(value, bool):
            return value

        value_str = value.lower().strip()
        if value_str in ("true", "yes", "1", "on"):
            return True

        if value_str in ("false", "no", "0", "off"):
            return False

        return default

    @overload
    def parse_ints(
        self,
        key: str,
        *,
        separator: str = ...,
        min_value: int | None = ...,
        max_value: int | None = ...,
    ) -> list[int] | None: ...

    @overload
    def parse_ints(
        self,
        key: str,
        default: list[int],
        *,
        separator: str = ...,
        min_value: int | None = ...,
        max_value: int | None = ...,
    ) -> list[int]: ...

    def parse_ints(
        self,
        key: str,
        default: list[int] | None = None,
        *,
        separator: str = ",",
        min_value: int | None = None,
        max_value: int | None = None,
    ) -> list[int] | None:
        """Parse a list of integers with optional range validation.

        Parameters
        ----------
        key : str
            Configuration key to look up
        default : list of int, optional
            Default value if key not found
        separator : str, optional
            String separator for list values
        min_value : int, optional
            Minimum allowed value (inclusive)
        max_value : int, optional
            Maximum allowed value (inclusive)

        Returns
        -------
        list of int or None
            List of parsed integers or default if not found/invalid

        Examples
        --------
        >>> parser = ConfigParser(cli_args={"ports": "80,443,8080"})
        >>> parser.parse_ints("ports")
        [80, 443, 8080]
        >>> parser.parse_ints("missing", default=[3000])
        [3000]
        >>> parser.parse_ints("invalid") is None
        True
        """

        value = self._get_value(key)
        if value is None:
            return default

        elif isinstance(value, bool):
            return default

        elif isinstance(value, (int, float)):
            return [int(value)]

        try:
            values = [int(v.strip()) for v in str(value).split(separator)]
            if min_value is not None:
                values = [v for v in values if v >= min_value]

            if max_value is not None:
                values = [v for v in values if v <= max_value]

            return values
        except (ValueError, TypeError):
            return default

    @overload
    def parse_floats(
        self,
        key: str,
        *,
        separator: str = ...,
        min_value: float | None = ...,
        max_value: float | None = ...,
    ) -> list[float] | None: ...

    @overload
    def parse_floats(
        self,
        key: str,
        default: list[float],
        *,
        separator: str = ...,
        min_value: float | None = ...,
        max_value: float | None = ...,
    ) -> list[float]: ...

    def parse_floats(
        self,
        key: str,
        default: list[float] | None = None,
        *,
        separator: str = ",",
        min_value: float | None = None,
        max_value: float | None = None,
    ) -> list[float] | None:
        """Parse a list of floats with optional range validation.

        Parameters
        ----------
        key : str
            Configuration key to look up
        default : list of float, optional
            Default value if key not found
        separator : str, optional
            String separator for list values
        min_value : float, optional
            Minimum allowed value (inclusive)
        max_value : float, optional
            Maximum allowed value (inclusive)

        Returns
        -------
        list of float or None
            List of parsed floats or default if not found/invalid

        Examples
        --------
        >>> parser = ConfigParser(cli_args={"ratios": "0.1,0.5,0.9"})
        >>> parser.parse_floats("ratios")
        [0.1, 0.5, 0.9]
        >>> parser.parse_floats("missing", default=[1.0])
        [1.0]
        >>> parser.parse_floats("missing") is None
        True
        """

        value = self._get_value(key)
        if value is None:
            return default

        elif isinstance(value, bool):
            return default

        elif isinstance(value, (int, float)):
            return [float(value)]

        try:
            values = [float(v.strip()) for v in str(value).split(separator)]

            if min_value is not None:
                values = [v for v in values if v >= min_value]

            if max_value is not None:
                values = [v for v in values if v <= max_value]

            return values
        except (ValueError, TypeError):
            return default

    @overload
    def parse_strings(
        self,
        key: str,
        *,
        separator: str = ...,
        allowed_values: list[str] | None = ...,
    ) -> list[str] | None: ...

    @overload
    def parse_strings(
        self,
        key: str,
        default: list[str],
        *,
        separator: str = ...,
        allowed_values: list[str] | None = ...,
    ) -> list[str]: ...

    def parse_strings(
        self,
        key: str,
        default: list[str] | None = None,
        *,
        separator: str = ",",
        allowed_values: list[str] | None = None,
    ) -> list[str] | None:
        """Parse a list of strings with optional allowed values validation.

        Parameters
        ----------
        key : str
            Configuration key to look up
        default : list of str, optional
            Default value if key not found
        separator : str, optional
            String separator for list values
        allowed_values : list of str, optional
            List of valid string values

        Returns
        -------
        list of str or None
            List of parsed strings or default if not found/invalid

        Examples
        --------
        >>> parser = ConfigParser(cli_args={"modes": "fast,slow"})
        >>> parser.parse_strings("modes")
        ['fast', 'slow']
        >>> parser.parse_strings("modes", allowed_values=["fast"])
        ['fast']
        >>> parser.parse_strings("missing", default=["medium"])
        ['medium']
        >>> parser.parse_strings("missing") is None
        True
        """

        value = self._get_value(key)
        if value is None:
            return default

        try:
            values = [v.strip() for v in str(value).split(separator)]
            if allowed_values is not None:
                values = [v for v in values if v in allowed_values]
            return values
        except (ValueError, TypeError):
            return default

    def _load_env_files(self, files: list[str | StringIO]) -> dict[str, str | None]:
        env_vars = {}

        # Load in reverse order so .env.local overrides .env
        for f in reversed(files):
            if isinstance(f, StringIO):
                env_vars.update(dotenv_values(stream=f))
            elif os.path.exists(f):
                env_vars.update(dotenv_values(f))

        # Add actual environment variables (higher priority than .env files)
        env_vars.update(os.environ)
        return env_vars

    def _get_value(self, key: str) -> str | int | float | bool | None:
        return self._get_cli_value(key) or self._get_env_value(key)

    def _get_cli_value(self, key: str) -> str | int | float | bool | None:
        cli_key = key.lower().replace("_", "-")
        return self._cli_args.get(cli_key)

    def _get_env_value(self, key: str) -> str | None:
        env_key = self._env_prefix + key.upper()
        return self._env_vars.get(env_key)


def _convert_value(value: Any, converter: Callable) -> Any:
    """Helper function to convert values with error handling."""
    if value is None:
        return None

    try:
        return converter(value)
    except (ValueError, TypeError):
        return None
