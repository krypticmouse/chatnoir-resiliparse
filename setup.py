# Copyright 2021 Janek Bevendorff
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import os
from Cython.Build import cythonize
import Cython.Compiler.Options
from setuptools import setup, Extension

Cython.Compiler.Options.annotate = bool(os.getenv('DEBUG'))

cpp_args = dict(
      extra_compile_args=['-std=c++17', '-O3', '-Wno-deprecated-declarations',
                          '-Wno-unreachable-code', '-Wno-unused-function'],
      extra_link_args=['-std=c++17', '-lz', '-llz4'])

# setup(
#       name='ResiliParse',
#       version='1.0',
#       description='Optimized and resilient web archive parsing library with fixed memory and execution time ceiling.',
#       author='Janek Bevendorff',
#       author_email='janek.bevendorff@uni-weimar.de',
#       url='https://webis.de',
#       license='Apache License 2.0',
#       packages=[],
# )

fastwarc_extensions = [
      Extension('fastwarc.warc', sources=['fastwarc/warc.pyx'], **cpp_args),
      Extension('fastwarc.stream_io', sources=['fastwarc/stream_io.pyx'], **cpp_args),
      Extension('fastwarc.tools', sources=['fastwarc/tools.pyx'], **cpp_args)
]

setup(
      name='FastWARC',
      version='1.0',
      description='Optimized WARC parsing library written in Cython',
      author='Janek Bevendorff',
      author_email='janek.bevendorff@uni-weimar.de',
      url='https://webis.de',
      license='Apache License 2.0',
      packages=['fastwarc'],
      install_requires=[
            'click',
            'tqdm'
      ],
      setup_requires=[
            'cython',
            'setuptools>=18.0'
      ],
      ext_modules=cythonize(fastwarc_extensions, annotate=Cython.Compiler.Options.annotate, language_level='3'),
      entry_points={
            'console_scripts': ['fastwarc=fastwarc.cli:main']
      }
)
