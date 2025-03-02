import fnmatch
from typing import Dict, List, Optional


class FilterManager:
    """
    Manages file pattern filters for controlling context lines in git diffs.
    """
    
    def __init__(self, filters: List[Dict] = None):
        """
        Initialize the FilterManager with a list of filter rules.
        
        Args:
            filters: List of filter dictionaries with 'file_pattern' and 'context_lines' keys.
                    If None, a default filter with 3 context lines will be used.
        """
        self.filters = filters or [{'file_pattern': '*', 'context_lines': 3}]
    
    def find_matching_rule(self, filename: str) -> Dict:
        """
        Find the first matching filter rule for a filename.
        
        Args:
            filename: The filename to match against filter patterns.
            
        Returns:
            The first matching filter rule, or a default rule with 3 context lines.
        """
        for filter_rule in self.filters:
            if fnmatch.fnmatch(filename, filter_rule['file_pattern']):
                return filter_rule
        return {'context_lines': 3}  # Default rule
    
    def apply_context_filter(self, hunks: List[Dict], context_lines: int) -> List[Dict]:
        """
        Adjust the context lines in hunks to match the specified number.
        
        Args:
            hunks: List of hunk dictionaries containing diff information.
            context_lines: Number of context lines to keep around changes.
            
        Returns:
            List of filtered hunks with adjusted context lines.
        """
        filtered_hunks = []
        
        for hunk in hunks:
            lines = hunk['lines']
            filtered_lines = []
            change_indices = []
            
            # First, find all the changed lines (+ or -)
            for i, line in enumerate(lines):
                if line.startswith('+') or line.startswith('-'):
                    change_indices.append(i)
            
            if not change_indices:
                continue
            
            # Now determine which context lines to keep
            lines_to_keep = set()
            for change_idx in change_indices:
                # Add the changed line
                lines_to_keep.add(change_idx)
                # Add context lines before
                for i in range(max(0, change_idx - context_lines), change_idx):
                    lines_to_keep.add(i)
                # Add context lines after
                for i in range(change_idx + 1, min(len(lines), change_idx + context_lines + 1)):
                    lines_to_keep.add(i)
            
            # Keep lines in their original order
            for i, line in enumerate(lines):
                if i in lines_to_keep:
                    filtered_lines.append(line)
            
            if filtered_lines:
                # Create a new hunk with all metadata preserved
                new_hunk = hunk.copy()
                new_hunk['lines'] = filtered_lines
                filtered_hunks.append(new_hunk)
        
        return filtered_hunks
    
    def post_process_files(self, patch_dict: Dict[str, List[Dict]]) -> Dict[str, List[Dict]]:
        """
        Apply post-processing filters to each file in the patch.
        
        Args:
            patch_dict: Dictionary mapping filenames to lists of hunks.
            
        Returns:
            Processed dictionary with filtered hunks.
        """
        processed_dict = {}
        
        for filename, hunks in patch_dict.items():
            rule = self.find_matching_rule(filename)
            
            # Check if this is a renamed file
            is_rename = any(hunk.get('is_rename', False) for hunk in hunks)
            
            # Apply context line filtering
            context_lines = rule.get('context_lines', 3)
            processed_hunks = self.apply_context_filter(hunks, context_lines)
            
            # Preserve rename information in processed hunks
            if is_rename and processed_hunks and hunks:
                for i, hunk in enumerate(processed_hunks):
                    if i < len(hunks):
                        hunk['is_rename'] = hunks[i].get('is_rename', False)
                        hunk['rename_from'] = hunks[i].get('rename_from')
                        hunk['rename_to'] = hunks[i].get('rename_to')
                        hunk['similarity_index'] = hunks[i].get('similarity_index')
            
            processed_dict[filename] = processed_hunks
        
        return processed_dict 