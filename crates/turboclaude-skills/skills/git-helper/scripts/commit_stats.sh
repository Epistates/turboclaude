#!/bin/bash

# Git Commit Statistics Analyzer
#
# Generates comprehensive commit statistics including:
# - Commits per author
# - Timeline analysis
# - Message quality metrics
# - Activity patterns
#
# Usage:
#   bash commit_stats.sh [--since="6 months ago"] [--until="now"] [--branch="main"]
#
# Options:
#   --since=DATE    Start date for analysis (default: 6 months ago)
#   --until=DATE    End date for analysis (default: now)
#   --branch=NAME   Branch to analyze (default: current branch)
#   --json          Output as JSON
#   --help          Show this help message

set -e

# Default values
SINCE="6 months ago"
UNTIL="now"
BRANCH=""
OUTPUT_JSON=false

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --since=*)
            SINCE="${1#*=}"
            shift
            ;;
        --until=*)
            UNTIL="${1#*=}"
            shift
            ;;
        --branch=*)
            BRANCH="${1#*=}"
            shift
            ;;
        --json)
            OUTPUT_JSON=true
            shift
            ;;
        --help)
            grep "^#" "$0" | grep -v "#!/bin/bash" | sed 's/^# //' | sed 's/^#//'
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "Error: Not a git repository" >&2
    exit 1
fi

# Use current branch if not specified
if [ -z "$BRANCH" ]; then
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
fi

# Function to print section header
print_header() {
    if [ "$OUTPUT_JSON" = false ]; then
        echo -e "\n${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BOLD}${CYAN}  $1${NC}"
        echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    fi
}

# Function to print subsection
print_subsection() {
    if [ "$OUTPUT_JSON" = false ]; then
        echo -e "\n${BOLD}$1${NC}"
        echo "────────────────────────────────────────────────────────────────────"
    fi
}

# Get total number of commits
TOTAL_COMMITS=$(git rev-list --count --since="$SINCE" --until="$UNTIL" "$BRANCH" 2>/dev/null || echo "0")

if [ "$TOTAL_COMMITS" -eq 0 ]; then
    echo "No commits found in the specified time range"
    exit 0
fi

# Main report
if [ "$OUTPUT_JSON" = false ]; then
    clear
    echo -e "${BOLD}${MAGENTA}"
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║                 GIT COMMIT STATISTICS REPORT                       ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    echo "Branch: $BRANCH"
    echo "Period: $SINCE to $UNTIL"
    echo "Total Commits: $TOTAL_COMMITS"
fi

# 1. COMMITS BY AUTHOR
print_header "COMMITS BY AUTHOR"

if [ "$OUTPUT_JSON" = false ]; then
    git log --since="$SINCE" --until="$UNTIL" --format='%aN' "$BRANCH" | \
        sort | uniq -c | sort -rn | \
        awk -v total="$TOTAL_COMMITS" '{
            percent = ($1 / total) * 100
            printf "  %-30s %5d commits  (%5.1f%%)\n", $2, $1, percent
        }'
fi

# 2. COMMITS TIMELINE
print_header "COMMITS TIMELINE"

print_subsection "Commits per Month"
if [ "$OUTPUT_JSON" = false ]; then
    git log --since="$SINCE" --until="$UNTIL" --format='%cd' --date=format:'%Y-%m' "$BRANCH" | \
        sort | uniq -c | \
        awk '{printf "  %-10s %5d commits  ", $2, $1; for(i=0;i<$1/5;i++) printf "▓"; printf "\n"}'
fi

print_subsection "Commits per Day of Week"
if [ "$OUTPUT_JSON" = false ]; then
    git log --since="$SINCE" --until="$UNTIL" --format='%cd' --date=format:'%A' "$BRANCH" | \
        sort | uniq -c | sort -k2 | \
        awk '{
            days["Monday"] = 1; days["Tuesday"] = 2; days["Wednesday"] = 3;
            days["Thursday"] = 4; days["Friday"] = 5; days["Saturday"] = 6; days["Sunday"] = 7;
        } {
            printf "  %-12s %5d commits  ", $2, $1;
            for(i=0;i<$1/3;i++) printf "▓";
            printf "\n"
        }'
fi

print_subsection "Commits by Hour of Day"
if [ "$OUTPUT_JSON" = false ]; then
    git log --since="$SINCE" --until="$UNTIL" --format='%cd' --date=format:'%H' "$BRANCH" | \
        sort -n | uniq -c | \
        awk '{
            hour = sprintf("%02d:00", $2)
            printf "  %-8s %4d commits  ", hour, $1;
            for(i=0;i<$1/2;i++) printf "▓";
            printf "\n"
        }'
fi

# 3. COMMIT MESSAGE ANALYSIS
print_header "COMMIT MESSAGE ANALYSIS"

# Count conventional commit types
FEAT_COUNT=$(git log --since="$SINCE" --until="$UNTIL" --oneline "$BRANCH" | grep -c "^[a-f0-9]* feat" || echo "0")
FIX_COUNT=$(git log --since="$SINCE" --until="$UNTIL" --oneline "$BRANCH" | grep -c "^[a-f0-9]* fix" || echo "0")
DOCS_COUNT=$(git log --since="$SINCE" --until="$UNTIL" --oneline "$BRANCH" | grep -c "^[a-f0-9]* docs" || echo "0")
REFACTOR_COUNT=$(git log --since="$SINCE" --until="$UNTIL" --oneline "$BRANCH" | grep -c "^[a-f0-9]* refactor" || echo "0")
TEST_COUNT=$(git log --since="$SINCE" --until="$UNTIL" --oneline "$BRANCH" | grep -c "^[a-f0-9]* test" || echo "0")
CHORE_COUNT=$(git log --since="$SINCE" --until="$UNTIL" --oneline "$BRANCH" | grep -c "^[a-f0-9]* chore" || echo "0")

CONVENTIONAL_TOTAL=$((FEAT_COUNT + FIX_COUNT + DOCS_COUNT + REFACTOR_COUNT + TEST_COUNT + CHORE_COUNT))

if [ "$OUTPUT_JSON" = false ]; then
    print_subsection "Commit Types (Conventional Commits)"

    if [ $CONVENTIONAL_TOTAL -gt 0 ]; then
        echo "  feat:      $FEAT_COUNT"
        echo "  fix:       $FIX_COUNT"
        echo "  docs:      $DOCS_COUNT"
        echo "  refactor:  $REFACTOR_COUNT"
        echo "  test:      $TEST_COUNT"
        echo "  chore:     $CHORE_COUNT"
        echo ""
        CONV_PERCENT=$(awk "BEGIN {printf \"%.1f\", ($CONVENTIONAL_TOTAL / $TOTAL_COMMITS) * 100}")
        echo "  Conventional commit compliance: $CONV_PERCENT%"
    else
        echo "  No conventional commits detected"
        echo "  Consider adopting conventional commit format!"
    fi

    print_subsection "Message Length Statistics"
    git log --since="$SINCE" --until="$UNTIL" --format='%s' "$BRANCH" | \
        awk '{
            len = length($0)
            if (len < 50) short++
            else if (len <= 72) good++
            else long++
            total++
        } END {
            printf "  Short (<50 chars):   %5d  (%5.1f%%)\n", short, (short/total)*100
            printf "  Good (50-72 chars):  %5d  (%5.1f%%)\n", good, (good/total)*100
            printf "  Long (>72 chars):    %5d  (%5.1f%%)\n", long, (long/total)*100
        }'
fi

# 4. CODE CHURN
print_header "CODE CHURN METRICS"

if [ "$OUTPUT_JSON" = false ]; then
    print_subsection "Total Changes"
    git log --since="$SINCE" --until="$UNTIL" --numstat --format="" "$BRANCH" | \
        awk '{
            add += $1
            del += $2
            files++
        } END {
            printf "  Lines added:      %8d\n", add
            printf "  Lines deleted:    %8d\n", del
            printf "  Net change:       %8d\n", add - del
            printf "  Files changed:    %8d\n", files
            if (files > 0) {
                printf "  Avg lines/file:   %8.1f\n", (add + del) / files
            }
        }'

    print_subsection "Top 10 Most Modified Files"
    git log --since="$SINCE" --until="$UNTIL" --name-only --format="" "$BRANCH" | \
        sort | uniq -c | sort -rn | head -10 | \
        awk '{printf "  %4d  %s\n", $1, $2}'
fi

# 5. COLLABORATION METRICS
print_header "COLLABORATION METRICS"

if [ "$OUTPUT_JSON" = false ]; then
    AUTHOR_COUNT=$(git log --since="$SINCE" --until="$UNTIL" --format='%aN' "$BRANCH" | sort -u | wc -l)
    AVG_COMMITS_PER_AUTHOR=$(awk "BEGIN {printf \"%.1f\", $TOTAL_COMMITS / $AUTHOR_COUNT}")

    # Calculate active days
    ACTIVE_DAYS=$(git log --since="$SINCE" --until="$UNTIL" --format='%cd' --date=short "$BRANCH" | sort -u | wc -l)

    # Calculate commit frequency
    DAYS_IN_PERIOD=$(( ($(date -d "$UNTIL" +%s) - $(date -d "$SINCE" +%s)) / 86400 ))
    if [ $DAYS_IN_PERIOD -eq 0 ]; then
        DAYS_IN_PERIOD=1
    fi
    COMMITS_PER_DAY=$(awk "BEGIN {printf \"%.2f\", $TOTAL_COMMITS / $DAYS_IN_PERIOD}")

    echo "  Total contributors:           $AUTHOR_COUNT"
    echo "  Avg commits per author:       $AVG_COMMITS_PER_AUTHOR"
    echo "  Active days:                  $ACTIVE_DAYS"
    echo "  Total days in period:         $DAYS_IN_PERIOD"
    echo "  Commits per day:              $COMMITS_PER_DAY"

    print_subsection "Most Prolific Authors (Top 5)"
    git log --since="$SINCE" --until="$UNTIL" --format='%aN' "$BRANCH" | \
        sort | uniq -c | sort -rn | head -5 | \
        awk '{printf "  %-30s %5d commits\n", $2, $1}'
fi

# 6. RECENT ACTIVITY
print_header "RECENT ACTIVITY"

if [ "$OUTPUT_JSON" = false ]; then
    print_subsection "Last 10 Commits"
    git log --since="$SINCE" --until="$UNTIL" -10 --format="%C(yellow)%h%C(reset) %C(blue)%an%C(reset) %s %C(green)(%cr)%C(reset)" "$BRANCH" | \
        sed 's/^/  /'
fi

# RECOMMENDATIONS
print_header "RECOMMENDATIONS"

if [ "$OUTPUT_JSON" = false ]; then
    echo ""

    # Check commit frequency
    if (( $(echo "$COMMITS_PER_DAY < 1" | bc -l) )); then
        echo -e "  ${YELLOW}⚠${NC}  Low commit frequency detected ($COMMITS_PER_DAY/day)"
        echo "     Consider committing more frequently for better tracking"
    else
        echo -e "  ${GREEN}✓${NC}  Good commit frequency ($COMMITS_PER_DAY commits/day)"
    fi

    # Check conventional commits
    if [ $CONVENTIONAL_TOTAL -eq 0 ]; then
        echo -e "  ${YELLOW}⚠${NC}  No conventional commits detected"
        echo "     Consider adopting conventional commit format"
        echo "     See: https://www.conventionalcommits.org/"
    elif [ $CONV_PERCENT ]; then
        if (( $(echo "$CONV_PERCENT < 50" | bc -l) )); then
            echo -e "  ${YELLOW}⚠${NC}  Low conventional commit compliance ($CONV_PERCENT%)"
            echo "     Aim for >80% conventional commit usage"
        else
            echo -e "  ${GREEN}✓${NC}  Good conventional commit compliance ($CONV_PERCENT%)"
        fi
    fi

    # Check author distribution
    if [ "$AUTHOR_COUNT" -eq 1 ]; then
        echo -e "  ${BLUE}ℹ${NC}  Single contributor detected"
        echo "     Consider code reviews and collaboration"
    else
        echo -e "  ${GREEN}✓${NC}  Multiple contributors ($AUTHOR_COUNT authors)"
    fi

    # Check for weekend commits
    WEEKEND_COMMITS=$(git log --since="$SINCE" --until="$UNTIL" --format='%cd' --date=format:'%A' "$BRANCH" | \
        grep -E "(Saturday|Sunday)" | wc -l)
    WEEKEND_PERCENT=$(awk "BEGIN {printf \"%.1f\", ($WEEKEND_COMMITS / $TOTAL_COMMITS) * 100}")

    if (( $(echo "$WEEKEND_PERCENT > 20" | bc -l) )); then
        echo -e "  ${YELLOW}⚠${NC}  High weekend activity ($WEEKEND_PERCENT%)"
        echo "     Consider work-life balance"
    fi

    echo ""
    echo -e "${BOLD}${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
fi

# JSON output (if requested)
if [ "$OUTPUT_JSON" = true ]; then
    # Build JSON output
    echo "{"
    echo "  \"summary\": {"
    echo "    \"branch\": \"$BRANCH\","
    echo "    \"since\": \"$SINCE\","
    echo "    \"until\": \"$UNTIL\","
    echo "    \"total_commits\": $TOTAL_COMMITS,"
    echo "    \"author_count\": $AUTHOR_COUNT,"
    echo "    \"commits_per_day\": $COMMITS_PER_DAY"
    echo "  },"
    echo "  \"commit_types\": {"
    echo "    \"feat\": $FEAT_COUNT,"
    echo "    \"fix\": $FIX_COUNT,"
    echo "    \"docs\": $DOCS_COUNT,"
    echo "    \"refactor\": $REFACTOR_COUNT,"
    echo "    \"test\": $TEST_COUNT,"
    echo "    \"chore\": $CHORE_COUNT"
    echo "  }"
    echo "}"
fi

exit 0
