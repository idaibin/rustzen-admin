# 📋 Changelog Maintenance Guide

## 🎯 New Workflow Overview

This project now uses a **git-cliff + manual optimization** hybrid workflow to maintain the changelog:

1. **Auto-generation**: Use git-cliff to generate the base changelog
2. **Manual optimization**: Add detailed descriptions for important features
3. **Regular updates**: Consolidate weekly or before releases

## 🛠️ Daily Operations Guide

### 1. View Current Unreleased Content

```bash
# View all changes since the last release
git cliff --unreleased
```

### 2. Update CHANGELOG.md

```bash
# Add Unreleased content to the beginning of CHANGELOG.md
git cliff --unreleased --prepend CHANGELOG.md
```

### 3. Generate Version-Specific Changelog

```bash
# Generate changes from v0.1.3 to v0.1.4
git cliff v0.1.3..v0.1.4

# Generate the complete changelog
git cliff --output CHANGELOG.md
```

## 📅 Maintenance Schedule

### Weekly Maintenance (Recommended: Friday)

1. **Run auto-generation**:

   ```bash
   git cliff --unreleased --prepend CHANGELOG.md
   ```

2. **Manually optimize important features**:

   - Major features like permission systems, architecture refactoring → Keep detailed descriptions
   - Daily bug fixes, UI adjustments → Use simplified format

3. **Review and cleanup**:
   - Merge duplicate entries
   - Adjust categories and order
   - Refine descriptions

### Pre-release Maintenance

1. **Generate release changelog**
2. **Move Unreleased to the specific version**
3. **Clear the Unreleased section**
4. **Create tag and push**

## 🎨 Format Standards

### Simplified Format (Daily Changes)

```markdown
### 🚀 Features

- **[backend]** Optimize user query API performance
- **[frontend]** Update login page styles
- **[system]** Batch adjust form validation logic
```

### Detailed Format (Major Features)

```markdown
### 🔐 Major Feature: Permission System Refactor

**Core Improvements**

- Added flexible permission checking mechanism
- Support for single, any, and all permission modes
- Memory cache optimization, response time reduced from 50ms to 0.1ms

**Migration Guide**

- Update route definitions...
- Adjust permission imports...
```

## 📝 Commit Message Standards

Ensure commit messages follow the conventional commit format:

```bash
# Features
feat(auth): add permission caching system
feat(system): implement user status management

# Fixes
fix(auth): handle token expiration correctly
fix(ui): resolve login form validation issue

# Documentation
docs(api): update authentication guide
docs: add deployment instructions

# Refactoring
refactor(core): simplify error handling logic
refactor(system): optimize database queries
```

## 🔧 Configuration Details

### cliff.toml Configuration File

- **Emoji categorization**: Automatically recognizes commit types and adds corresponding emojis
- **Filter rules**: Skips unimportant chore commits
- **Format template**: Unified changelog format

### Common Configuration Modifications

```toml
# Modify commit categorization
commit_parsers = [
    { message = "^feat", group = "🚀 Features"},
    { message = "^fix", group = "🐛 Bug Fixes"},
    # ... other categories
]

# Skip specific commits
{ message = "^chore\\(deps.*\\)", skip = true},
```

## 🎯 Best Practices

1. **Update regularly**: Don’t accumulate too many unrecorded changes
2. **Clear categorization**: Ensure each change has an appropriate category
3. **Accurate descriptions**: Concise but information-complete descriptions
4. **User perspective**: Describe changes from the user’s point of view
5. **Consistency**: Unified format and language style

---

**Remember**: Use tools to generate the base version, and manual optimization to improve quality!
