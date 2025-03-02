"""
Setup script for RepoDiff.
"""
from setuptools import setup, find_packages

setup(
    name="repodiff",
    version="0.2.0",
    description="A tool for generating optimized git diffs for LLM analysis",
    author="RepoDiff Team",
    packages=find_packages(),
    install_requires=[
        "tiktoken",
    ],
    entry_points={
        "console_scripts": [
            "repodiff=repodiff.cli:main",
        ],
    },
    python_requires=">=3.6",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
    ],
) 