extern crate replicante;

use replicante::ReplicanteMain;


fn main() {
    let replicante = ReplicanteMain::configure();
    replicante.run();
}
