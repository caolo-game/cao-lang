from setuptools import setup
from setuptools_rust import Binding, RustExtension

with open("README.md", "r") as f:
    long_description = f.read()

setup(
    name="cao-lang",
    version="0.1.14",
    description="The node based 'language' that governs the actors of the game Cao-Lo",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Daniel Kiss",
    author_email="littlesnorrboy@gmail.com",
    rust_extensions=[
        RustExtension("cao_lang.cao_lang_py", "py/Cargo.toml", binding=Binding.PyO3)
    ],
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Development Status :: 2 - Pre-Alpha",
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Rust",
    ],
    packages=["cao_lang"],
    package_dir={"": "py"},
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
    url="https://github.com/caolo-game/cao-lang",
    project_urls={
        "Bug Tracker": "https://github.com/caolo-game/cao-lang/issues",
    },
)
