# A Zephyr application exported by ArcLang (arclang export -f zephyr).
#   west build -b <board> .                       the controller
#   west build -b <board> . -- -DARCLANG_MAIN=pil_io      processor in the loop, through the I/O
#   west build -b <board> . -- -DARCLANG_MAIN=pil_direct -DARCLANG_PERIOD_US=10000
#                                                  processor in the loop, the step() API
cmake_minimum_required(VERSION 3.20.0)

if(EXISTS ${CMAKE_CURRENT_SOURCE_DIR}/arclang_module/zephyr/module.yml)
  list(APPEND ZEPHYR_EXTRA_MODULES ${CMAKE_CURRENT_SOURCE_DIR}/arclang_module)
endif()

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(arclang_app C)

include(${CMAKE_CURRENT_SOURCE_DIR}/arclang_app.cmake)
set(ARCLANG_MAIN "main" CACHE STRING "main, pil_io or pil_direct")

file(GLOB arclang_model_sources ${CMAKE_CURRENT_SOURCE_DIR}/src/model/*.c)
file(GLOB arclang_glue_sources ${CMAKE_CURRENT_SOURCE_DIR}/src/glue/*.c)
target_include_directories(app PRIVATE src src/model src/glue)
target_sources(app PRIVATE ${arclang_model_sources} src/arclang_io_zephyr.c src/${ARCLANG_MAIN}.c)
if(ARCLANG_MAIN STREQUAL "pil_direct")
  # The step() API alone: no glue, the case's step as the period.
  target_compile_definitions(app PRIVATE ARCLANG_DIRECT=1)
else()
  target_sources(app PRIVATE ${arclang_glue_sources})
endif()
if(ARCLANG_MAIN MATCHES "^pil")
  target_sources(app PRIVATE src/arclang_pil.c src/pil_link.c)
endif()
if(DEFINED ARCLANG_PERIOD_US)
  target_compile_definitions(app PRIVATE ${ARCLANG_PERIOD_MACRO}=${ARCLANG_PERIOD_US}UL)
endif()
