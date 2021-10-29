name: Chore Issue
description: File a chore issue
title: "[主题 issue] "
labels: ["chore"]
body:
  - type: textarea
    id: description
    attributes:
      label: 背景描述
      value: "- owner: \n- cooperator: \n- reviewer: \n\n"
  - type: textarea
    id: plan
    attributes:
      label: 方案设计
      value: "- owner: \n- cooperator: \n- reviewer: \n\n"
  - type: textarea
    id: deployment
    attributes:
      label: 方案落实
      value: "- owner: \n- cooperator: \n- reviewer: \n\n"
  - type: textarea
    id: link
    attributes:
      label: 相关链接
