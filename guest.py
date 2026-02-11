import wit_world
from componentize_py_types import Err
import json


def handle(e: Exception) -> Err[str]:
    message = str(e)
    if message == "":
        return Err(f"{type(e).__name__}")
    else:
        return Err(f"{type(e).__name__}: {message}")


class WitWorld(wit_world.WitWorld):
    def eval(self, code: str) -> str:
        try:
            program = compile(code, "<string>", "eval")
            return json.dumps(eval(program))
        except Exception as e:
            raise handle(e)

    def exec(self, code: str) -> None:
        try:
            local_vars = {}

            # Split into lines and filter empty ones, but keep track of indentation
            all_lines = code.split('\n')

            # Group lines into complete statements (handling multi-line blocks)
            statements = []
            i = 0
            while i < len(all_lines):
                line = all_lines[i]

                # Skip empty lines
                if not line.strip():
                    i += 1
                    continue

                # Start of a new statement
                current_stmt = [line]

                # Check if this line ends with ':' (start of indented block)
                if line.rstrip().endswith(':'):
                    i += 1
                    # Collect all indented lines that follow
                    while i < len(all_lines):
                        next_line = all_lines[i]
                        if not next_line.strip():
                            # Keep empty lines in the block
                            current_stmt.append(next_line)
                        elif next_line[0] == ' ' or next_line[0] == '\t':
                            # Indented line - part of the block
                            current_stmt.append(next_line)
                        else:
                            # Not indented - end of block
                            break
                        i += 1
                else:
                    i += 1

                statements.append('\n'.join(current_stmt))

            if not statements:
                return json.dumps(None)

            # Execute all but the last statement
            for stmt in statements[:-1]:
                exec(stmt, {}, local_vars)

            # Try to evaluate last statement as expression
            last_stmt = statements[-1]
            try:
                result = eval(last_stmt, {}, local_vars)
            except SyntaxError:
                exec(last_stmt, {}, local_vars)
                result = None

            return json.dumps(result)
        except Exception as e:
            raise handle(e)
