mod widget;
mod widgets;

use taffy::prelude::*;

use taffy::prelude::*;

pub fn taffy_test() -> Result<(), taffy::TaffyError> {
    let mut taffy: TaffyTree<()> = TaffyTree::new();

    // left
    let child_t1 = taffy.new_leaf(Style {
        size: Size {
            width: Dimension::Length(5.0),
            height: Dimension::Length(5.0),
        },
        margin: Rect {
            left: LengthPercentageAuto::Length(10.0),
            right: LengthPercentageAuto::Length(10.0),
            top: LengthPercentageAuto::Length(10.0),
            bottom: LengthPercentageAuto::Length(10.0),
        },
        ..Default::default()
    })?;

    let div1 = taffy.new_with_children(
        Style {
            size: Size {
                width: Dimension::Percent(0.5),
                height: Dimension::Percent(1.0),
            },
            margin: Rect {
                left: LengthPercentageAuto::Length(10.0),
                right: LengthPercentageAuto::Length(10.0),
                top: LengthPercentageAuto::Length(10.0),
                bottom: LengthPercentageAuto::Length(10.0),
            },
            // justify_content: JustifyContent::Center,
            ..Default::default()
        },
        &[child_t1],
    )?;

    // right
    let child_t2 = taffy.new_leaf(Style {
        size: Size {
            width: Dimension::Length(5.0),
            height: Dimension::Length(5.0),
        },
        ..Default::default()
    })?;

    let div2 = taffy.new_with_children(
        Style {
            size: Size {
                width: Dimension::Percent(0.5),
                height: Dimension::Percent(1.0),
            },
            // justify_content: JustifyContent::Center,
            ..Default::default()
        },
        &[child_t2],
    )?;

    let container = taffy.new_with_children(
        Style {
            size: Size {
                width: Dimension::Percent(1.0),
                height: Dimension::Percent(1.0),
            },
            ..Default::default()
        },
        &[div1, div2],
    )?;

    taffy.compute_layout(
        container,
        Size {
            height: AvailableSpace::Definite(100.0),
            width: AvailableSpace::Definite(100.0),
        },
    )?;

    println!("node: {:#?}", taffy.layout(container));

    println!("div1: {:#?}", taffy.layout(div1));
    println!("div2: {:#?}", taffy.layout(div2));

    println!("child1: {:#?}", taffy.layout(child_t1));
    println!("child2: {:#?}", taffy.layout(child_t2));

    taffy.print_tree(container);

    Ok(())
}
