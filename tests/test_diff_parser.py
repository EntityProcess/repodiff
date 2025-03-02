import pytest
from repodiff.utils.diff_parser import DiffParser


class TestDiffParser:
    """
    Tests for the DiffParser class.
    """
    
    def test_parse_unified_diff_empty(self):
        """Test parsing an empty diff."""
        diff_output = ""
        result = DiffParser.parse_unified_diff(diff_output)
        assert result == {}
    
    def test_parse_unified_diff_single_file(self):
        """Test parsing a diff with a single file."""
        diff_output = """diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3"""
        
        result = DiffParser.parse_unified_diff(diff_output)
        
        assert len(result) == 1
        assert 'file1.txt' in result
        assert len(result['file1.txt']) == 1  # One hunk
        
        hunk = result['file1.txt'][0]
        assert hunk['old_start'] == 1
        assert hunk['old_count'] == 3
        assert hunk['new_start'] == 1
        assert hunk['new_count'] == 3
        assert hunk['lines'] == [' line1', '-line2', '+line2_modified', ' line3']
    
    def test_parse_unified_diff_multiple_files(self):
        """Test parsing a diff with multiple files."""
        diff_output = """diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3
diff --git a/file2.txt b/file2.txt
--- a/file2.txt
+++ b/file2.txt
@@ -1,2 +1,3 @@
 line1
+line2_added
 line3"""
        
        result = DiffParser.parse_unified_diff(diff_output)
        
        assert len(result) == 2
        assert 'file1.txt' in result
        assert 'file2.txt' in result
        
        # Check file1.txt
        assert len(result['file1.txt']) == 1
        assert result['file1.txt'][0]['lines'] == [' line1', '-line2', '+line2_modified', ' line3']
        
        # Check file2.txt
        assert len(result['file2.txt']) == 1
        assert result['file2.txt'][0]['lines'] == [' line1', '+line2_added', ' line3']
    
    def test_parse_unified_diff_multiple_hunks(self):
        """Test parsing a diff with multiple hunks in a file."""
        diff_output = """diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3
@@ -10,2 +10,3 @@
 line10
+line11_added
 line12"""
        
        result = DiffParser.parse_unified_diff(diff_output)
        
        assert len(result) == 1
        assert 'file1.txt' in result
        assert len(result['file1.txt']) == 2  # Two hunks
        
        # Check first hunk
        assert result['file1.txt'][0]['old_start'] == 1
        assert result['file1.txt'][0]['old_count'] == 3
        assert result['file1.txt'][0]['lines'] == [' line1', '-line2', '+line2_modified', ' line3']
        
        # Check second hunk
        assert result['file1.txt'][1]['old_start'] == 10
        assert result['file1.txt'][1]['old_count'] == 2
        assert result['file1.txt'][1]['lines'] == [' line10', '+line11_added', ' line12']
    
    def test_parse_unified_diff_with_rename(self):
        """Test parsing a diff with a renamed file."""
        diff_output = """diff --git a/old_file.txt b/new_file.txt
similarity index 90%
rename from old_file.txt
rename to new_file.txt
--- a/old_file.txt
+++ b/new_file.txt
@@ -1,3 +1,3 @@
 line1
-line2
+line2_modified
 line3"""
        
        result = DiffParser.parse_unified_diff(diff_output)
        
        assert len(result) == 1
        assert 'new_file.txt' in result
        
        hunk = result['new_file.txt'][0]
        assert hunk['is_rename'] == True
        assert hunk['rename_from'] == 'old_file.txt'
        assert hunk['rename_to'] == 'new_file.txt'
        assert hunk['similarity_index'] == 'similarity index 90%'
    
    def test_reconstruct_patch_empty(self):
        """Test reconstructing an empty patch."""
        patch_dict = {}
        result = DiffParser.reconstruct_patch(patch_dict)
        assert result == ""
    
    def test_reconstruct_patch_single_file(self):
        """Test reconstructing a patch with a single file."""
        patch_dict = {
            'file1.txt': [
                {
                    'header': '@@ -1,3 +1,3 @@',
                    'lines': [' line1', '-line2', '+line2_modified', ' line3']
                }
            ]
        }
        
        result = DiffParser.reconstruct_patch(patch_dict)
        
        expected = """diff --git a/file1.txt b/file1.txt
--- a/file1.txt
+++ b/file1.txt
 line1
-line2
+line2_modified
 line3"""
        
        assert result == expected
    
    def test_reconstruct_patch_with_rename(self):
        """Test reconstructing a patch with a renamed file."""
        patch_dict = {
            'new_file.txt': [
                {
                    'header': '@@ -1,3 +1,3 @@',
                    'lines': [' line1', '-line2', '+line2_modified', ' line3'],
                    'is_rename': True,
                    'rename_from': 'old_file.txt',
                    'rename_to': 'new_file.txt',
                    'similarity_index': 'similarity index 90%'
                }
            ]
        }
        
        result = DiffParser.reconstruct_patch(patch_dict)
        
        expected = """diff --git a/old_file.txt b/new_file.txt
similarity index 90%
rename from old_file.txt
rename to new_file.txt
--- a/old_file.txt
+++ b/new_file.txt
 line1
-line2
+line2_modified
 line3"""
        
        assert result == expected 