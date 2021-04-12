from .cao_lang_py import *


def version_human():
    """
    Human readable version of this library
    """
    from importlib.metadata import version

    nat = native_version()
    pack = version("cao_lang")

    return f"Native version: {nat}\nPython package version: {pack}"
