import pytest
from repodiff.filters.filter_manager import FilterManager


class TestFilterManager:
    """
    Tests for the FilterManager class.
    """
    
    def test_init_with_default_filters(self):
        """Test initialization with default filters."""
        filter_manager = FilterManager()
        assert filter_manager.filters == [{'file_pattern': '*', 'context_lines': 3}]
    
    def test_init_with_custom_filters(self):
        """Test initialization with custom filters."""
        custom_filters = [
            {'file_pattern': '*.cs', 'context_lines': 5},
            {'file_pattern': '*.py', 'context_lines': 2}
        ]
        filter_manager = FilterManager(custom_filters)
        assert filter_manager.filters == custom_filters
    
    def test_find_matching_rule_exact_match(self):
        """Test finding a matching rule with an exact match."""
        filters = [
            {'file_pattern': '*.cs', 'context_lines': 5},
            {'file_pattern': '*.py', 'context_lines': 2}
        ]
        filter_manager = FilterManager(filters)
        
        rule = filter_manager.find_matching_rule('test.cs')
        assert rule == {'file_pattern': '*.cs', 'context_lines': 5}
    
    def test_find_matching_rule_no_match(self):
        """Test finding a matching rule with no match."""
        filters = [
            {'file_pattern': '*.cs', 'context_lines': 5},
            {'file_pattern': '*.py', 'context_lines': 2}
        ]
        filter_manager = FilterManager(filters)
        
        rule = filter_manager.find_matching_rule('test.js')
        assert rule == {'context_lines': 3}  # Default rule
    
    def test_find_matching_rule_multiple_matches(self):
        """Test finding a matching rule with multiple potential matches."""
        filters = [
            {'file_pattern': '*.cs', 'context_lines': 5},
            {'file_pattern': '*Test*.cs', 'context_lines': 10},
            {'file_pattern': '*.py', 'context_lines': 2}
        ]
        filter_manager = FilterManager(filters)
        
        # Should return the first match
        rule = filter_manager.find_matching_rule('TestClass.cs')
        assert rule == {'file_pattern': '*.cs', 'context_lines': 5}
    
    def test_apply_context_filter_empty_hunks(self):
        """Test applying context filter to empty hunks."""
        filter_manager = FilterManager()
        
        filtered_hunks = filter_manager.apply_context_filter([], 3)
        assert filtered_hunks == []
    
    def test_apply_context_filter_no_changes(self):
        """Test applying context filter to hunks with no changes."""
        filter_manager = FilterManager()
        
        hunks = [
            {
                'header': '@@ -1,3 +1,3 @@',
                'lines': [' line1', ' line2', ' line3']
            }
        ]
        
        filtered_hunks = filter_manager.apply_context_filter(hunks, 3)
        assert filtered_hunks == []
    
    def test_apply_context_filter_with_changes(self):
        """Test applying context filter to hunks with changes."""
        filter_manager = FilterManager()
        
        hunks = [
            {
                'header': '@@ -1,5 +1,5 @@',
                'lines': [' line1', ' line2', '-line3', '+line3_modified', ' line4', ' line5']
            }
        ]
        
        filtered_hunks = filter_manager.apply_context_filter(hunks, 1)
        
        # Should keep the changed lines and 1 line of context before and after
        expected_lines = [' line2', '-line3', '+line3_modified', ' line4']
        assert len(filtered_hunks) == 1
        assert filtered_hunks[0]['lines'] == expected_lines
    
    def test_post_process_files(self):
        """Test post-processing files with different filters."""
        filters = [
            {'file_pattern': '*.cs', 'context_lines': 2},
            {'file_pattern': '*.py', 'context_lines': 1}
        ]
        filter_manager = FilterManager(filters)
        
        patch_dict = {
            'file1.cs': [
                {
                    'header': '@@ -1,5 +1,5 @@',
                    'lines': [' line1', ' line2', '-line3', '+line3_modified', ' line4', ' line5']
                }
            ],
            'file2.py': [
                {
                    'header': '@@ -1,5 +1,5 @@',
                    'lines': [' line1', ' line2', '-line3', '+line3_modified', ' line4', ' line5']
                }
            ]
        }
        
        processed_dict = filter_manager.post_process_files(patch_dict)
        
        # CS file should have 2 lines of context
        assert len(processed_dict['file1.cs'][0]['lines']) == 6  # 2 changes + 4 context lines
        
        # PY file should have 1 line of context
        assert len(processed_dict['file2.py'][0]['lines']) == 4  # 2 changes + 2 context lines
    
    def test_post_process_files_with_rename(self):
        """Test post-processing files with rename information."""
        filter_manager = FilterManager()
        
        patch_dict = {
            'new_file.cs': [
                {
                    'header': '@@ -1,3 +1,3 @@',
                    'lines': [' line1', '-line2', '+line2_modified'],
                    'is_rename': True,
                    'rename_from': 'old_file.cs',
                    'rename_to': 'new_file.cs',
                    'similarity_index': 'similarity index 90%'
                }
            ]
        }
        
        processed_dict = filter_manager.post_process_files(patch_dict)
        
        # Check that rename information is preserved
        assert processed_dict['new_file.cs'][0]['is_rename'] == True
        assert processed_dict['new_file.cs'][0]['rename_from'] == 'old_file.cs'
        assert processed_dict['new_file.cs'][0]['rename_to'] == 'new_file.cs'
        assert processed_dict['new_file.cs'][0]['similarity_index'] == 'similarity index 90%' 