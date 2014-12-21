import {bootstrap, Component, Decorator, TemplateConfig, NgElement} from 'core/core';

// Angular 2.0 supports 3 basic types of directives:
// - Component - the basic building blocks of Angular 2.0 apps. Backed by
//   ShadowDom.(http://www.html5rocks.com/en/tutorials/webcomponents/shadowdom/)
// - Decorator - add behavior to existing elements.
// - Template - allow for stamping out of a html template (not in this demo).

// @Component is AtScript syntax to annotate the HelloCmp class as an Angular
// 2.0 component.
@Component({
  // The Selector prop tells Angular on which elements to instantiate this
  // class. The syntax supported is a basic subset of CSS selectors, for example
  // 'element', '[attr]', [attr=foo]', etc.
  selector: 'hello-app',
  // These are services that would be created if a class in the component's
  // template tries to inject them.
  componentServices: [GreetingService],
  template: new TemplateConfig({
    // The template for the component.
    // Expressions in the template (like {{greeting}}) are evaluated in the
    // context of the HelloCmp class below.
    inline: `{{greeting}} <span red>world</span>!`,
    // All directives used in the template need to be specified. This allows for
    // modularity (RedDec can only be used in this template)
    // and better tooling (the template can be invalidated if the attribute is
    // misspelled).
    directives: [RedDec]
  })
})
class HelloCmp {
  greeting: string;
  constructor(service: GreetingService) {
    this.greeting = service.greeting;
  }
}

// Decorators are light-weight. They don't allow for templates, or new
// expression contexts (use @Component or @Template for those needs).
@Decorator({
  selector: '[red]'
})
class RedDec {
  // NgElement is always injectable and it wraps the element on which the
  // directive was found by the compiler.
  constructor(el: NgElement) {
    el.domElement.style.color = 'red';
  }
}

// A service used by the HelloCmp component.
class GreetingService {
  greeting:string;
  constructor() {
    this.greeting = 'hello';
  }
}

export function main() {
  // Bootstrapping only requires specifying a root component.
  // The boundary between the Angular application and the rest of the page is
  // the shadowDom of this root component.
  // The selector of the component passed in is used to find where to insert the
  // application.
  // You can use the light dom of the <hello-app> tag as temporary content (for
  // example 'Loading...') before the application is ready.
  bootstrap(HelloCmp);
}
