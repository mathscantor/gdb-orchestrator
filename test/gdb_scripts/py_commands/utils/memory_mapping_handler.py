import gdb
import re
from typing import *
import sys
import os
import json
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from messenger import Severity, Messenger


class MemoryMappingHandler:

    def __init__(self):
        self.__messenger = Messenger()
        self.__header_index_lookup = dict()
        self.__init_header()
        self.__unknown_section_prefix = "unknown_section_"
        return

    def __init_header(self):
        __header_raw = gdb.execute("info proc mappings", to_string=True).split("\n")[3]
        index = 0
        if "Start Addr" in __header_raw:
            self.__header_index_lookup["start_addr"] = index
            index += 1

        if "End Addr" in __header_raw:
            self.__header_index_lookup["end_addr"] = index
            index += 1

        if "Size" in __header_raw:
            self.__header_index_lookup["size"] = index
            index += 1

        if "Offset" in __header_raw:
            self.__header_index_lookup["offset"] = index
            index += 1

        if "Perms" in __header_raw:
            self.__header_index_lookup["perms"] = index
            index += 1

        if "objfile" in __header_raw:
            self.__header_index_lookup["objfile"] = index
            index += 1
        return

    def get_memory_sections(self) -> Dict[str, Dict[str, str]]:

        '''
        Memory Section JSON format:
        {
            "/home/user/test_binaries/test": {
                "start_addr": "0x55680ce23000",
                "end_addr": "0x55680ce23fff"
            },
            "[heap]": {
                "start_addr": "0x55680ecd7000",
                "end_addr": "0x55680ecf7fff"
            },
            "/usr/lib/x86_64-linux-gnu/libc.so.6": {
                "start_addr": "0x7fe3c8b25000",
                "end_addr": "0x7fe3c8b26fff"
            },
            .
            .
            .
        }
        '''
        unknown_section_num = 1
        memory_sections = dict()
        header_index_lookup_size = len(self.__header_index_lookup)
        start_addr_index = self.__header_index_lookup["start_addr"]
        end_addr_index = self.__header_index_lookup["end_addr"]
        objfile_addr_index = self.__header_index_lookup["objfile"]

        memory_mappings = gdb.execute("info proc mappings", to_string=True).split("\n")[4:-1]

        for memory_map in memory_mappings:
            map_tokens = re.split(r"\s+", memory_map.strip())

            # If there is no objfile, add a "unknown_section_X" to it
            if len(map_tokens) != header_index_lookup_size:
                map_tokens.insert(objfile_addr_index, "{}{}".format(self.__unknown_section_prefix,
                                                                    unknown_section_num))
                unknown_section_num += 1

            # Add binary path as key if we haven't already
            if map_tokens[objfile_addr_index] not in memory_sections:
                memory_sections[map_tokens[objfile_addr_index]] = {"start_addr": map_tokens[start_addr_index],
                                                                   "end_addr": map_tokens[end_addr_index]}

            # Note that linux maps memory by [x, y) format. Inclusive of x but excluding y.
            # See Here: https://stackoverflow.com/questions/34819167/gdb-find-command-error-warning-unable-to-access-x-bytes-of-target-memory-at-y
            memory_sections[map_tokens[objfile_addr_index]]["end_addr"] = hex(int(map_tokens[end_addr_index], 16) - 1)
        # print(json.dumps(memory_sections, indent=4))
        return memory_sections

    @property
    def unknown_section_prefix(self) -> str:
        return self.__unknown_section_prefix

    @property
    def header_index_lookup(self) -> Dict[int, str]:
        return self.__header_index_lookup
