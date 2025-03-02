import re
from typing import Dict, List


class DiffParser:
    """
    Parser for git diff output that converts it to a structured format.
    """
    
    @staticmethod
    def parse_unified_diff(diff_output: str) -> Dict[str, List[Dict]]:
        """
        Parse the unified diff output into a dictionary of files and their hunks.
        
        Args:
            diff_output: The raw output from git diff command.
            
        Returns:
            Dictionary mapping filenames to lists of hunks.
        """
        files = {}
        current_file = None
        current_hunks = []
        lines = diff_output.split('\n')
        is_rename = False
        rename_from = None
        rename_to = None
        similarity_index = None
        
        i = 0
        while i < len(lines):
            line = lines[i]
            
            if line.startswith('diff --git'):
                # Save previous file data if exists
                if current_file:
                    files[current_file] = current_hunks
                
                current_file = None
                current_hunks = []
                is_rename = False
                rename_from = None
                rename_to = None
                similarity_index = None
                
                # Check for rename by looking ahead
                j = i + 1
                while j < len(lines) and not lines[j].startswith('diff --git'):
                    if lines[j].startswith('similarity index '):
                        similarity_index = lines[j]
                        is_rename = True
                    elif lines[j].startswith('rename from '):
                        rename_from = lines[j][12:]  # Remove 'rename from ' prefix
                    elif lines[j].startswith('rename to '):
                        rename_to = lines[j][10:]  # Remove 'rename to ' prefix
                    j += 1
                    
            elif line.startswith('--- a/'):
                # For renames, we need to handle this differently
                if not is_rename:
                    i += 1
                    continue
            elif line.startswith('+++ b/'):
                if is_rename and rename_from and rename_to:
                    current_file = rename_to
                else:
                    current_file = line[6:]  # Remove '+++ b/' prefix
            elif line.startswith('@@'):
                # Parse hunk header
                match = re.match(r'@@ -(\d+),?(\d+)? \+(\d+),?(\d+)? @@', line)
                if match:
                    old_start = int(match.group(1))
                    old_count = int(match.group(2) or 1)
                    new_start = int(match.group(3))
                    new_count = int(match.group(4) or 1)
                    current_hunks.append({
                        'header': line,
                        'old_start': old_start,
                        'old_count': old_count,
                        'new_start': new_start,
                        'new_count': new_count,
                        'lines': [],
                        'is_rename': is_rename,
                        'rename_from': rename_from,
                        'rename_to': rename_to,
                        'similarity_index': similarity_index
                    })
            elif current_file and current_hunks:
                current_hunks[-1]['lines'].append(line)
            
            i += 1
        
        if current_file:
            files[current_file] = current_hunks
        
        return files
    
    @staticmethod
    def reconstruct_patch(patch_dict: Dict[str, List[Dict]]) -> str:
        """
        Reconstruct a unified diff from the processed patch dictionary.
        
        Args:
            patch_dict: Dictionary mapping filenames to lists of hunks.
            
        Returns:
            Reconstructed unified diff as a string.
        """
        output = []
        
        for filename, hunks in patch_dict.items():
            # Check if any hunks have rename information
            is_rename = any(hunk.get('is_rename', False) for hunk in hunks)
            rename_from = None
            rename_to = None
            similarity_index = None
            
            if is_rename and hunks:
                # Get rename information from the first hunk
                rename_from = hunks[0].get('rename_from')
                rename_to = hunks[0].get('rename_to')
                similarity_index = hunks[0].get('similarity_index')
                
                # Construct the rename diff header
                output.append(f'diff --git a/{rename_from} b/{rename_to}')
                if similarity_index:
                    output.append(similarity_index)
                output.append(f'rename from {rename_from}')
                output.append(f'rename to {rename_to}')
                output.append(f'--- a/{rename_from}')
                output.append(f'+++ b/{rename_to}')
            else:
                # Regular file diff
                output.append(f'diff --git a/{filename} b/{filename}')
                output.append(f'--- a/{filename}')
                output.append(f'+++ b/{filename}')
            
            for hunk in hunks:
                # Skip the hunk header as it's not necessary for understanding changes
                # output.append(hunk['header'])
                output.extend(hunk['lines'])
        
        return '\n'.join(output) 