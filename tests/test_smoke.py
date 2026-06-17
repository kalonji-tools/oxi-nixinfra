"""Smoke test — verify all public types are importable."""


def test_public_exports():
    import oxi_nixinfra

    for name in oxi_nixinfra.__all__:
        assert getattr(oxi_nixinfra, name) is not None, (
            f"__all__ declares '{name}' but the module exposes None"
            " — check __init__.py re-exports"
        )
