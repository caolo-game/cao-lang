from setuptools import setup
from setuptools_rust import Binding, RustExtension


setup(
    name="cao-lang",
    version="0.1.4",
    rust_extensions=[
        RustExtension("cao_lang.cao_lang_py", "py/Cargo.toml", binding=Binding.PyO3)
    ],
    packages=["cao_lang"],
    package_dir={"": "py"},
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
