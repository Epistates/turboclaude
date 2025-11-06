#!/usr/bin/env python3
"""
Git Branch Analyzer

Analyzes all branches in a Git repository and identifies stale branches,
branch statistics, and provides recommendations for branch management.

Usage:
    python analyze_branches.py [--days=30] [--remote] [--json]

Options:
    --days=N     Consider branches stale after N days (default: 30)
    --remote     Include remote branches in analysis
    --json       Output results as JSON
"""

import argparse
import json
import subprocess
import sys
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple


class BranchInfo:
    """Represents information about a Git branch."""

    def __init__(self, name: str, last_commit_date: datetime,
                 last_commit_sha: str, last_commit_msg: str, author: str):
        self.name = name
        self.last_commit_date = last_commit_date
        self.last_commit_sha = last_commit_sha
        self.last_commit_msg = last_commit_msg
        self.author = author
        self.commit_count = 0
        self.is_merged = False
        self.ahead_behind: Optional[Tuple[int, int]] = None

    def to_dict(self) -> Dict:
        """Convert branch info to dictionary."""
        return {
            'name': self.name,
            'last_commit_date': self.last_commit_date.isoformat(),
            'last_commit_sha': self.last_commit_sha,
            'last_commit_msg': self.last_commit_msg,
            'author': self.author,
            'commit_count': self.commit_count,
            'is_merged': self.is_merged,
            'ahead_behind': self.ahead_behind,
            'days_since_update': self.days_since_update()
        }

    def days_since_update(self) -> int:
        """Calculate days since last update."""
        return (datetime.now() - self.last_commit_date).days


class GitBranchAnalyzer:
    """Analyzes Git repository branches."""

    def __init__(self, stale_days: int = 30, include_remote: bool = False):
        self.stale_days = stale_days
        self.include_remote = include_remote
        self.current_branch = self._get_current_branch()
        self.default_branch = self._get_default_branch()

    def _run_git_command(self, args: List[str]) -> str:
        """Execute a git command and return output."""
        try:
            result = subprocess.run(
                ['git'] + args,
                capture_output=True,
                text=True,
                check=True
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError as e:
            print(f"Error running git command: {e}", file=sys.stderr)
            print(f"stderr: {e.stderr}", file=sys.stderr)
            return ""

    def _get_current_branch(self) -> str:
        """Get the current branch name."""
        return self._run_git_command(['rev-parse', '--abbrev-ref', 'HEAD'])

    def _get_default_branch(self) -> str:
        """Determine the default branch (main or master)."""
        branches = self._run_git_command(['branch', '-a']).split('\n')
        for branch in branches:
            branch = branch.strip().replace('* ', '')
            if 'main' in branch:
                return 'main'
        return 'master'

    def get_branches(self) -> List[str]:
        """Get list of all branches."""
        if self.include_remote:
            flag = '-a'
        else:
            flag = '-l'

        output = self._run_git_command(['branch', flag])
        branches = []

        for line in output.split('\n'):
            line = line.strip().replace('* ', '').replace('remotes/', '')
            if line and '->' not in line:  # Skip symbolic references
                branches.append(line)

        return branches

    def get_branch_info(self, branch: str) -> Optional[BranchInfo]:
        """Get detailed information about a branch."""
        try:
            # Get last commit info
            format_str = '%H|%aI|%an|%s'
            commit_info = self._run_git_command([
                'log', '-1', f'--format={format_str}', branch
            ])

            if not commit_info:
                return None

            sha, date_str, author, msg = commit_info.split('|', 3)
            commit_date = datetime.fromisoformat(date_str.replace('Z', '+00:00'))

            # Get commit count
            commit_count = self._run_git_command([
                'rev-list', '--count', branch
            ])

            # Check if merged into default branch
            merged_output = self._run_git_command([
                'branch', '--merged', self.default_branch
            ])
            is_merged = any(branch in line for line in merged_output.split('\n'))

            info = BranchInfo(branch, commit_date, sha, msg, author)
            info.commit_count = int(commit_count) if commit_count else 0
            info.is_merged = is_merged

            # Get ahead/behind info relative to default branch
            if branch != self.default_branch:
                try:
                    ahead_behind = self._run_git_command([
                        'rev-list', '--left-right', '--count',
                        f'{self.default_branch}...{branch}'
                    ])
                    if ahead_behind:
                        behind, ahead = ahead_behind.split()
                        info.ahead_behind = (int(ahead), int(behind))
                except Exception:
                    info.ahead_behind = None

            return info

        except Exception as e:
            print(f"Error getting info for branch {branch}: {e}", file=sys.stderr)
            return None

    def analyze(self) -> Dict:
        """Perform complete branch analysis."""
        branches = self.get_branches()
        branch_infos = []

        for branch in branches:
            info = self.get_branch_info(branch)
            if info:
                branch_infos.append(info)

        # Sort by last commit date (newest first)
        branch_infos.sort(key=lambda x: x.last_commit_date, reverse=True)

        # Categorize branches
        stale_branches = [b for b in branch_infos
                         if b.days_since_update() > self.stale_days]
        active_branches = [b for b in branch_infos
                          if b.days_since_update() <= self.stale_days]
        merged_branches = [b for b in branch_infos if b.is_merged]

        return {
            'summary': {
                'total_branches': len(branch_infos),
                'active_branches': len(active_branches),
                'stale_branches': len(stale_branches),
                'merged_branches': len(merged_branches),
                'current_branch': self.current_branch,
                'default_branch': self.default_branch,
                'stale_threshold_days': self.stale_days
            },
            'stale_branches': [b.to_dict() for b in stale_branches],
            'active_branches': [b.to_dict() for b in active_branches],
            'merged_branches': [b.to_dict() for b in merged_branches]
        }


def print_text_report(analysis: Dict):
    """Print human-readable analysis report."""
    summary = analysis['summary']

    print("\n" + "="*70)
    print("  GIT BRANCH ANALYSIS REPORT")
    print("="*70)

    print(f"\nCurrent Branch: {summary['current_branch']}")
    print(f"Default Branch: {summary['default_branch']}")
    print(f"Stale Threshold: {summary['stale_threshold_days']} days")

    print("\n" + "-"*70)
    print("SUMMARY")
    print("-"*70)
    print(f"Total Branches:   {summary['total_branches']}")
    print(f"Active Branches:  {summary['active_branches']}")
    print(f"Stale Branches:   {summary['stale_branches']}")
    print(f"Merged Branches:  {summary['merged_branches']}")

    # Stale branches section
    if analysis['stale_branches']:
        print("\n" + "-"*70)
        print(f"STALE BRANCHES (>{summary['stale_threshold_days']} days)")
        print("-"*70)

        for branch in analysis['stale_branches']:
            days = branch['days_since_update']
            name = branch['name']
            author = branch['author']
            last_commit = branch['last_commit_msg'][:50]
            merged_status = "✓ merged" if branch['is_merged'] else "✗ not merged"

            print(f"\n• {name}")
            print(f"  Last updated: {days} days ago")
            print(f"  Author: {author}")
            print(f"  Status: {merged_status}")
            print(f"  Last commit: {last_commit}")

            if branch['ahead_behind']:
                ahead, behind = branch['ahead_behind']
                print(f"  Position: {ahead} ahead, {behind} behind")

    # Recommendations
    print("\n" + "-"*70)
    print("RECOMMENDATIONS")
    print("-"*70)

    deletable = [b for b in analysis['stale_branches'] if b['is_merged']]
    if deletable:
        print(f"\n✓ {len(deletable)} branches can be safely deleted (stale + merged):")
        for branch in deletable[:5]:  # Show first 5
            print(f"  - {branch['name']}")
        if len(deletable) > 5:
            print(f"  ... and {len(deletable) - 5} more")

    stale_unmerged = [b for b in analysis['stale_branches'] if not b['is_merged']]
    if stale_unmerged:
        print(f"\n⚠ {len(stale_unmerged)} stale branches are NOT merged:")
        for branch in stale_unmerged[:5]:
            print(f"  - {branch['name']}")
        if len(stale_unmerged) > 5:
            print(f"  ... and {len(stale_unmerged) - 5} more")
        print("\n  Review these branches before deleting!")

    # Most active authors
    if analysis['active_branches']:
        authors = {}
        for branch in analysis['active_branches']:
            author = branch['author']
            authors[author] = authors.get(author, 0) + 1

        print(f"\n📊 Most active contributors:")
        for author, count in sorted(authors.items(), key=lambda x: x[1], reverse=True)[:5]:
            print(f"  - {author}: {count} active branch(es)")

    print("\n" + "="*70 + "\n")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Analyze Git branches and identify stale branches'
    )
    parser.add_argument(
        '--days',
        type=int,
        default=30,
        help='Consider branches stale after N days (default: 30)'
    )
    parser.add_argument(
        '--remote',
        action='store_true',
        help='Include remote branches in analysis'
    )
    parser.add_argument(
        '--json',
        action='store_true',
        help='Output results as JSON'
    )

    args = parser.parse_args()

    # Check if we're in a git repository
    try:
        subprocess.run(
            ['git', 'rev-parse', '--git-dir'],
            capture_output=True,
            check=True
        )
    except subprocess.CalledProcessError:
        print("Error: Not a git repository", file=sys.stderr)
        sys.exit(1)

    # Perform analysis
    analyzer = GitBranchAnalyzer(
        stale_days=args.days,
        include_remote=args.remote
    )

    try:
        analysis = analyzer.analyze()

        if args.json:
            print(json.dumps(analysis, indent=2))
        else:
            print_text_report(analysis)

    except Exception as e:
        print(f"Error during analysis: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
