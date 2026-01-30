#!/usr/bin/env python3
"""
Git Branch Search (gbs) - A tool to search and checkout git branches by substring
"""

import os
import sys
import subprocess
import re
import argparse
from typing import List, Optional, Tuple


# ANSI color codes for terminal output
class Colors:
    HEADER = "\033[95m"
    BLUE = "\033[94m"
    CYAN = "\033[96m"
    GREEN = "\033[92m"
    YELLOW = "\033[93m"
    RED = "\033[91m"
    ENDC = "\033[0m"
    BOLD = "\033[1m"
    UNDERLINE = "\033[4m"


def print_header(text: str) -> None:
    """Print a formatted header."""
    print(f"\n{Colors.HEADER}{Colors.BOLD}=== {text} ==={Colors.ENDC}\n")


def print_step(text: str) -> None:
    """Print a step in the process."""
    print(f"\n{Colors.BLUE}→ {text}{Colors.ENDC}")


def print_success(text: str) -> None:
    """Print a success message."""
    print(f"\n{Colors.GREEN}✓ {text}{Colors.ENDC}")


def print_error(text: str) -> None:
    """Print an error message."""
    print(f"\n{Colors.RED}✗ {text}{Colors.ENDC}")


def run_git_command(command: List[str]) -> Tuple[bool, str]:
    """Run a git command and return success status and output."""
    try:
        result = subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        return True, result.stdout
    except subprocess.CalledProcessError as e:
        return False, e.stderr


def is_git_repository() -> bool:
    """Check if the current directory is a git repository."""
    success, _ = run_git_command(["git", "rev-parse", "--is-inside-work-tree"])
    return success


def update_remote_branches() -> bool:
    """Update remote branches using git remote update."""
    print_step("Updating remote branches...")
    success, output = run_git_command(["git", "remote", "update", "origin", "--prune"])
    if success:
        print_success("Remote branches updated")
    else:
        print_error(f"Failed to update remote branches: {output}")
    return success


def get_all_branches() -> List[str]:
    """Get all branches (local and remote)."""
    success, output = run_git_command(["git", "branch", "-a"])
    if not success:
        print_error("Failed to get branches")
        return []

    branches = []
    for line in output.splitlines():
        branch = line.strip()
        # Remove the asterisk from the current branch
        if branch.startswith("*"):
            branch = branch[1:].strip()
        branches.append(branch)

    print_step(f"Found {len(branches)} branches")
    return branches


def search_branches(
    query: str, branches: List[str], exact_match: bool = False
) -> List[str]:
    """Search for branches containing the query string or matching exactly (case-insensitive)."""
    matches = []
    query_lower = query.lower()

    for branch_name in branches:
        # For substring searches, we maintain the original behavior of generally not showing
        # 'origin/' prefixed branches directly in results, as per the script's original notes/help text.
        # This promotes finding local or easily understandable branch names first.
        # For exact matches, this filter is bypassed to allow finding specific remote branches.
        if not exact_match and branch_name.startswith("origin/"):
            # However, if the query itself starts with "origin/", we should not skip in substring mode either
            if not query_lower.startswith("origin/"):
                continue

        if exact_match:
            if query_lower == branch_name.lower():
                matches.append(branch_name)
        else:
            if query_lower in branch_name.lower():
                matches.append(branch_name)

    return list(set(matches))  # Deduplicate results


def simplify_branch_name(branch: str) -> str:
    """Simplify branch names by removing remotes/origin/ prefix."""
    # Remove 'remotes/' prefix which appears in some git outputs
    if branch.startswith("remotes/"):
        branch = branch[len("remotes/") :]
    # Remove 'origin/' prefix to simplify display
    if branch.startswith("origin/"):
        branch = branch[len("origin/") :]
    return branch


def display_branch_selection(matches: List[str]) -> Optional[Tuple[str, str]]:
    """Display a list of matching branches and prompt for selection.
    Returns a tuple of (original_branch_name, simplified_branch_name) if a branch is selected.
    """
    if not matches:
        # Caller (main) now handles the 'no matches found' message and re-prompt logic.
        return None

    # Create a list of (original_name, simplified_name) pairs
    branch_pairs = [(branch, simplify_branch_name(branch)) for branch in matches]

    # Deduplicate based on simplified names
    unique_branches = []
    seen_simplified = set()
    for orig, simp in branch_pairs:
        if simp not in seen_simplified:
            unique_branches.append((orig, simp))
            seen_simplified.add(simp)

    if len(unique_branches) == 1:
        orig, simp = unique_branches[0]
        print_success(f"Found single matching branch: {simp}")
        return orig, simp

    print_header("Matching Branches")

    for i, (orig, simp) in enumerate(unique_branches, 1):
        current_indicator = ""
        success, output = run_git_command(["git", "branch", "--show-current"])
        if success and output.strip() == simp.strip():
            current_indicator = f" {Colors.YELLOW}(current){Colors.ENDC}"
        print(f"{Colors.CYAN}{i}.{Colors.ENDC} {simp}{current_indicator}")

    while True:
        try:
            choice = input(
                f"\n{Colors.BOLD}Enter number to checkout (or 'q' to quit): {Colors.ENDC}"
            )
            if choice.lower() == "q":
                return None

            index = int(choice) - 1
            if 0 <= index < len(unique_branches):
                return unique_branches[index]
            else:
                print_error("Invalid selection. Please try again.")
        except ValueError:
            print_error("Please enter a valid number or 'q' to quit.")


def checkout_branch(branch: str, simplified_name: str) -> bool:
    """Checkout the selected branch properly, handling remote branches by creating local tracking branches."""
    print_step(f"Checking out branch '{simplified_name}'...")

    # Check if this is a remote branch
    is_remote = branch.startswith("remotes/origin/") or branch.startswith("origin/")

    if is_remote:
        # For remote branches, create a local tracking branch
        local_branch = simplified_name
        # Check if local branch with this name already exists
        success, output = run_git_command(
            ["git", "show-ref", "--verify", f"refs/heads/{local_branch}"]
        )

        if success:
            # Local branch exists, just check it out
            success, output = run_git_command(["git", "checkout", local_branch])
        else:
            # Create a new local branch that tracks the remote branch
            success, output = run_git_command(
                ["git", "checkout", "-b", local_branch, "--track", branch]
            )
    else:
        # For local branches, just check them out directly
        success, output = run_git_command(["git", "checkout", branch])

    if success:
        print_success(f"Successfully checked out '{simplified_name}'")
    else:
        print_error(f"Failed to checkout branch: {output}")

    return success


def print_help():
    """Print help information about the script."""
    help_text = f"""
{Colors.HEADER}{Colors.BOLD}Git Branch Search (gbs){Colors.ENDC}

{Colors.BOLD}Description:{Colors.ENDC}
  A tool to search and checkout git branches by substring or exact name.
  
{Colors.BOLD}Usage:{Colors.ENDC}
  gbs [options]
  
{Colors.BOLD}Options:{Colors.ENDC}
  -h, --help    Show this help message and exit
  -e, --exact   Perform an exact match for the branch name (case-insensitive)
  
{Colors.BOLD}Interactive Usage:{Colors.ENDC}
  1. Run the command in a git repository.
  2. Optionally, use -e for an exact match.
  3. Enter a substring (or exact name if -e is used) to search for branches.
  4. If no branches are found, you'll be prompted to try another search.
  5. Select a branch from the list by number.
  6. The selected branch will be checked out.
  
{Colors.BOLD}Notes:{Colors.ENDC}
  - The script will update remote branches before searching.
  - Substring search primarily shows local-like branches (e.g., 'feature/xyz', not 'origin/feature/xyz' unless query starts with 'origin/').
  - Exact match (-e) allows matching any branch name, including those like 'origin/feature/xyz'.
    """
    print(help_text)


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="Git Branch Search (gbs) - Search and checkout git branches",
        add_help=False,
    )
    parser.add_argument(
        "-h", "--help", action="store_true", help="Show this help message and exit"
    )
    parser.add_argument(
        "-e",
        "--exact",
        action="store_true",
        help="Perform an exact match for the branch name (case-insensitive)",
    )
    return parser.parse_known_args()


def main():
    """Main function to run the git branch search tool."""
    args, _ = parse_args()

    if args.help:
        print_help()
        sys.exit(0)

    print_header("Git Branch Search")

    if not is_git_repository():
        print_error("Not in a git repository")
        sys.exit(1)

    if not update_remote_branches():
        pass  # Continue anyway, as this is not critical

    all_branches_cache = None

    while True:
        search_type_msg = "exact name" if args.exact else "substring"
        query_prompt = f"\n{Colors.BOLD}Enter branch {search_type_msg} to search for (or 'q' to quit): {Colors.ENDC}"
        query = input(query_prompt)

        if not query.strip() or query.lower() == "q":
            print_step("Search cancelled or no query provided. Exiting.")
            sys.exit(0)

        if all_branches_cache is None:
            print_step("Fetching all branches...")
            all_branches_cache = get_all_branches()
            if not all_branches_cache:
                print_error(
                    "Failed to retrieve branch list. This can happen in an empty repository. Exiting."
                )
                sys.exit(1)

        matches = search_branches(query, all_branches_cache, args.exact)

        if not matches:
            print_error(
                f"No branches found matching '{query}' with {search_type_msg} search."
            )
            retry_choice = input(
                f"{Colors.BOLD}Try another search? (y/n): {Colors.ENDC}"
            ).lower()
            if retry_choice == "y":
                continue
            else:
                print_step("Exiting.")
                sys.exit(0)

        selection = display_branch_selection(matches)

        if selection:
            original_branch, simplified_name = selection
            checkout_branch(original_branch, simplified_name)
            break
        else:
            # This case is reached if user quits the selection prompt in display_branch_selection
            print_step("Branch selection aborted by user. Exiting.")
            sys.exit(0)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nOperation cancelled by user")
        sys.exit(1)
