-- LuaNext API definitions for ZeroBrane Studio auto-completion
-- This file provides type information for LuaNext built-in types and standard library

return {
  -- Built-in primitive types
  string = {
    type = "type",
    description = "String type representing text",
  },

  number = {
    type = "type",
    description = "Number type representing numeric values",
  },

  boolean = {
    type = "type",
    description = "Boolean type representing true/false values",
  },

  unknown = {
    type = "type",
    description = "Unknown type - type-safe alternative to any",
  },

  void = {
    type = "type",
    description = "Void type for functions that return nothing",
  },

  never = {
    type = "type",
    description = "Never type for values that never occur",
  },

  any = {
    type = "type",
    description = "Any type - bypasses type checking",
  },

  null = {
    type = "value",
    description = "Null value representing absence of value",
  },

  undefined = {
    type = "value",
    description = "Undefined value",
  },

  -- Generic types
  Array = {
    type = "class",
    description = "Generic array type",
    childs = {
      length = {
        type = "value",
        description = "Number of elements in the array",
      },
      push = {
        type = "method",
        args = "(element: T)",
        returns = "void",
        description = "Add an element to the end of the array",
      },
      pop = {
        type = "method",
        args = "()",
        returns = "T | undefined",
        description = "Remove and return the last element",
      },
      map = {
        type = "method",
        args = "(callback: (value: T, index: number) => U)",
        returns = "Array<U>",
        description = "Create a new array with the results of calling a function on every element",
      },
      filter = {
        type = "method",
        args = "(callback: (value: T, index: number) => boolean)",
        returns = "Array<T>",
        description = "Create a new array with elements that pass the test",
      },
      forEach = {
        type = "method",
        args = "(callback: (value: T, index: number) => void)",
        returns = "void",
        description = "Execute a function for each element",
      },
    },
  },

  Record = {
    type = "class",
    description = "Generic record/dictionary type",
  },

  Partial = {
    type = "class",
    description = "Make all properties of a type optional",
  },

  Required = {
    type = "class",
    description = "Make all properties of a type required",
  },

  Readonly = {
    type = "class",
    description = "Make all properties of a type readonly",
  },

  Pick = {
    type = "class",
    description = "Construct a type by picking properties from another type",
  },

  Omit = {
    type = "class",
    description = "Construct a type by omitting properties from another type",
  },

  -- Type keywords
  type = {
    type = "keyword",
    description = "Define a type alias",
  },

  interface = {
    type = "keyword",
    description = "Define an interface",
  },

  namespace = {
    type = "keyword",
    description = "Define a namespace",
  },

  enum = {
    type = "keyword",
    description = "Define an enumeration",
  },

  declare = {
    type = "keyword",
    description = "Declare ambient types or values",
  },

  -- Import/Export keywords
  import = {
    type = "keyword",
    description = "Import modules or types",
  },

  export = {
    type = "keyword",
    description = "Export values or types",
  },

  from = {
    type = "keyword",
    description = "Specify module source in import statements",
  },

  as = {
    type = "keyword",
    description = "Rename imports or type assertions",
  },

  default = {
    type = "keyword",
    description = "Default export",
  },

  -- Modifiers
  readonly = {
    type = "keyword",
    description = "Mark properties as read-only",
  },

  public = {
    type = "keyword",
    description = "Public access modifier",
  },

  private = {
    type = "keyword",
    description = "Private access modifier",
  },

  protected = {
    type = "keyword",
    description = "Protected access modifier",
  },

  static = {
    type = "keyword",
    description = "Static member modifier",
  },

  abstract = {
    type = "keyword",
    description = "Abstract class or member",
  },

  -- Variable declarations
  const = {
    type = "keyword",
    description = "Declare a constant",
  },

  let = {
    type = "keyword",
    description = "Declare a block-scoped variable",
  },

  var = {
    type = "keyword",
    description = "Declare a function-scoped variable",
  },
}
