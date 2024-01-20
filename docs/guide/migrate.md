# Migrate to 0.5

Due to a major change in August syntax between 0.4.3 and 0.5,
this guide exists to show the conversions between the two.

## Tasks

<div class="grid" markdown>
```title="<0.5"
Task build {

}
```

```august title=">=0.5"
unit Build {

}
```
</div>

## Dependencies

<div class="grid" markdown>
```title="<0.5"
Task build:[test] {

}
```

```august title=">=0.5"
unit Build {
    depends_on(Test)
}
```
</div>

<div class="grid" markdown>
```title="<0.5"
Task build:[test, clean] {

}
```

```august title=">=0.5"
unit Build {
    depends_on(Test, Clean)
}
```
</div>

## Command Definitions

<div class="grid" markdown>
```title="<0.5"
Task build {
    lints;
}

cmddef lints {
    
}
```

```august title=">=0.5"
unit Build {
    do(Lints)
}

unit Lints {

}
```
</div>


## Pragma

<div class="grid" markdown>
```title="<0.5"
#pragma build build
#pragma test test
```

```august title=">=0.5"
expose Build as build
expose Test as test
```
</div>

## Exec

<div class="grid" markdown>
```title="<0.5"
exec("cargo build --release");
~("cargo install --path .");
```

```august title=">=0.5"
exec(cargo build --release)
~(cargo install --path .)
```
</div>

