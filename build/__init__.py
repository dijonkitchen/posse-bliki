"""posse-bliki build package.

Public entry points: ``build_site`` and ``default_config``.
The implementation is intentionally small and lives entirely in ``build.py``.
This module is replaceable — the durable artifacts are ``spec/`` and ``tests/``.
"""
from .build import build_site, default_config, BuildError

__all__ = ["build_site", "default_config", "BuildError"]
